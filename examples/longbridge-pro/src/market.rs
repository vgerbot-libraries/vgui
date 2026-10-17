//! Simulated market + brokerage model for the Longbridge Pro terminal.
//!
//! All data is generated in-process from a tiny deterministic PRNG so the
//! terminal is fully offline and reproducible on both native and WASM. A
//! fixed-interval timer drives [`MarketState::tick`], which random-walks
//! every quote. Orders fill immediately at the live mid price against a
//! cash balance; opposing fills net against an open position and realize
//! P/L.

/// One tradable instrument.
#[derive(Clone)]
pub struct Instrument {
    pub symbol: &'static str,
    pub name: &'static str,
    pub sector: &'static str,
    pub price: f64,
    pub prev_close: f64,
    pub history: Vec<f64>,
    pub day_high: f64,
    pub day_low: f64,
    pub volume: u64,
    pub market_cap: f64,
    pub volatility: f64,
}

impl Instrument {
    /// Percentage change from the previous session close.
    pub fn change_pct(&self) -> f64 {
        (self.price - self.prev_close) / self.prev_close * 100.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Side {
    Long,
    Short,
}

/// An open position in one instrument.
#[derive(Clone)]
pub struct Position {
    pub symbol: String,
    pub side: Side,
    pub qty: f64,
    pub avg_price: f64,
    pub realized: f64,
}



#[derive(Clone, Copy, PartialEq, Eq)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    pub fn label(self) -> &'static str {
        match self {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        }
    }
}

/// A filled market-order record in the history.
#[derive(Clone)]
pub struct Order {
    pub symbol: String,
    pub side: OrderSide,
    pub qty: f64,
    pub price: f64,
}

#[derive(Clone)]
pub struct MarketState {
    pub instruments: Vec<Instrument>,
    pub positions: Vec<Position>,
    pub orders: Vec<Order>,
    pub cash: f64,
    rng: Rng,
}

/// Minimal deterministic xorshift PRNG (no platform RNG, no `getrandom`).
#[derive(Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng {
            state: if seed == 0 { 0x9E37_79B9_7F4A_7C15 } else { seed },
        }
    }

    /// Uniform f64 in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        (x >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Standard-normal sample via the Box–Muller transform.
    pub fn next_normal(&mut self) -> f64 {
        let u1 = self.next_f64().max(1e-9);
        let u2 = self.next_f64();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

const HISTORY_LEN: usize = 160;

impl MarketState {
    pub fn new() -> Self {
        let mut rng = Rng::new(20261017);

        // (symbol, name, sector, base price, per-tick volatility)
        let quotes: &[(&str, &str, &str, f64, f64)] = &[
            ("AAPL", "Apple Inc.", "Technology", 189.42, 0.011),
            ("TSLA", "Tesla, Inc.", "Automotive", 246.85, 0.028),
            ("NVDA", "NVIDIA Corp.", "Semiconductors", 131.28, 0.022),
            ("700.HK", "Tencent Holdings", "Internet", 388.60, 0.015),
            ("9988.HK", "Alibaba Group", "E-Commerce", 107.30, 0.019),
            ("3690.HK", "Meituan", "Local Services", 168.90, 0.024),
            ("0005.HK", "HSBC Holdings", "Banking", 64.20, 0.009),
            ("BRK.B", "Berkshire Hathaway", "Financials", 462.10, 0.008),
        ];

        let mut instruments = Vec::new();
        for (symbol, name, sector, base, vol) in quotes {
            // Build the series so it always terminates at the current price.
            let mut history = vec![*base; HISTORY_LEN];
            let mut p = *base;
            for i in (0..HISTORY_LEN - 1).rev() {
                let shock = rng.next_normal() * vol * base * 0.5;
                let drift = (base - p) * 0.02;
                p -= drift + shock;
                p = p.max(base * 0.5);
                history[i] = p;
            }
            history[HISTORY_LEN - 1] = *base;

            let prev_close = history[HISTORY_LEN - 30];
            instruments.push(Instrument {
                symbol,
                name,
                sector,
                price: *base,
                prev_close,
                day_high: history.iter().cloned().fold(f64::MIN, f64::max) * 1.002,
                day_low: history.iter().cloned().fold(f64::MAX, f64::min) * 0.998,
                volume: (rng.next_f64() * 40.0 + 5.0) as u64 * 1_000_000,
                market_cap: base * (rng.next_f64() * 140.0 + 2.0) * 1e9 * 0.01,
                history,
                volatility: *vol,
            });
        }

        MarketState {
            instruments,
            positions: Vec::new(),
            orders: Vec::new(),
            cash: 1_000_000.0,
            rng,
        }
    }

    pub fn instrument(&self, symbol: &str) -> Option<&Instrument> {
        self.instruments.iter().find(|i| i.symbol == symbol)
    }

    /// Advance every quote by one random-walk tick.
    pub fn tick(&mut self) {
        for inst in &mut self.instruments {
            let shock = self.rng.next_normal() * inst.volatility * inst.price * 0.12;
            let mean_revert = (inst.prev_close - inst.price) * 0.004;
            let next = (inst.price + shock + mean_revert).max(inst.price * 0.5);
            inst.price = next;
            inst.day_high = inst.day_high.max(next);
            inst.day_low = inst.day_low.min(next);
            inst.history.push(next);
            if inst.history.len() > HISTORY_LEN {
                inst.history.remove(0);
            }
        }
    }

    fn signed_cash_flow(side: OrderSide, qty: f64, price: f64) -> f64 {
        match side {
            OrderSide::Buy => -qty * price,
            OrderSide::Sell => qty * price,
        }
    }

    /// Fill a market order immediately at the instrument's live price.
    /// Returns `false` if the instrument is unknown.
    pub fn place_order(&mut self, symbol: &str, side: OrderSide, qty: f64) -> bool {
        let price = match self.instrument(symbol) {
            Some(i) => i.price,
            None => return false,
        };

        self.cash += Self::signed_cash_flow(side, qty, price);
        self.orders.insert(
            0,
            Order {
                symbol: symbol.to_string(),
                side,
                qty,
                price,
            },
        );
        self.apply_fill(symbol, side, qty, price);
        true
    }

    fn apply_fill(&mut self, symbol: &str, side: OrderSide, qty: f64, price: f64) {
        let (open_side, close_side) = match side {
            // A buy closes a short first, then opens/extends a long.
            OrderSide::Buy => (Side::Long, Side::Short),
            // A sell closes a long first, then opens/extends a short.
            OrderSide::Sell => (Side::Short, Side::Long),
        };

        let mut remaining = qty;

        // Net against an opposing position.
        if let Some(pos) = self
            .positions
            .iter_mut()
            .find(|p| p.symbol == symbol && p.side == close_side)
        {
            let closed = remaining.min(pos.qty);
            let pnl = match close_side {
                // Closing a short: sold high (avg), bought back low (price).
                Side::Short => (pos.avg_price - price) * closed,
                // Closing a long: sold at price, opened at avg.
                Side::Long => (price - pos.avg_price) * closed,
            };
            pos.realized += pnl;
            pos.qty -= closed;
            remaining -= closed;
        }

        // Sweep any fully-closed opposing rows.
        self.positions.retain(|p| p.qty > 1e-9);

        if remaining > 1e-9 {
            // Merge into an existing same-side position, else open one.
            if let Some(pos) = self
                .positions
                .iter_mut()
                .find(|p| p.symbol == symbol && p.side == open_side)
            {
                let total = pos.qty + remaining;
                pos.avg_price = (pos.avg_price * pos.qty + price * remaining) / total;
                pos.qty = total;
            } else {
                self.positions.push(Position {
                    symbol: symbol.to_string(),
                    side: open_side,
                    qty: remaining,
                    avg_price: price,
                    realized: 0.0,
                });
            }
        }
    }

    /// Total realized + unrealized P/L across open positions.
    pub fn total_pnl(&self) -> f64 {
        self.positions
            .iter()
            .map(|p| {
                let unrealized = match self.instrument(&p.symbol) {
                    Some(inst) => match p.side {
                        Side::Long => (inst.price - p.avg_price) * p.qty,
                        Side::Short => (p.avg_price - inst.price) * p.qty,
                    },
                    None => 0.0,
                };
                p.realized + unrealized
            })
            .sum()
    }

    /// Net liquidation value: cash plus mark-to-market of open positions.
    pub fn equity(&self) -> f64 {
        let exposure: f64 = self
            .positions
            .iter()
            .map(|p| {
                self.instrument(&p.symbol)
                    .map(|i| i.price * p.qty)
                    .unwrap_or(0.0)
            })
            .sum();
        self.cash + exposure
    }
}

/// Format helpers shared across the views.
pub fn money(v: f64) -> String {
    format!("{:.2}", v)
}

pub fn signed_money(v: f64) -> String {
    if v >= 0.0 {
        format!("+{:.2}", v)
    } else {
        format!("{:.2}", v)
    }
}

pub fn pct(v: f64) -> String {
    format!("{:+.2}%", v)
}

pub fn compact_usd(v: f64) -> String {
    if v.abs() >= 1e12 {
        format!("${:.2}T", v / 1e12)
    } else if v.abs() >= 1e9 {
        format!("${:.2}B", v / 1e9)
    } else if v.abs() >= 1e6 {
        format!("${:.1}M", v / 1e6)
    } else {
        format!("${:.0}", v)
    }
}

pub fn volume(v: u64) -> String {
    if v >= 1_000_000 {
        format!("{:.1}M", v as f64 / 1e6)
    } else if v >= 1_000 {
        format!("{:.1}K", v as f64 / 1e3)
    } else {
        format!("{}", v)
    }
}
