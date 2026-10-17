#![cfg_attr(target_family = "wasm", no_main)]

mod chart;
mod market;

use gpui::{px, size, App, Bounds, IntoElement, WindowBounds, WindowOptions};
use vgui::for_each;
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

use market::{
    compact_usd, money, pct, signed_money, volume, Instrument, MarketState, OrderSide, Side,
};

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;
#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

// ── Terminal palette (Longbridge dark) ────────────────────────────────
const UP: &str = "#f23645"; // Asian-market convention: red up
const DOWN: &str = "#2ebd85"; // green down
const BRAND: &str = "#ff7a45";

fn change_color(v: f64) -> &'static str {
    if v >= 0.0 {
        UP
    } else {
        DOWN
    }
}

/// Text span colored at runtime (`css!` colors are compile-time only).
fn tinted(text: String, hex: &str) -> gpui::AnyElement {
    gpui::div()
        .text_color(color(hex))
        .child(text)
        .into_any_element()
}

fn fmt_qty(q: f64) -> String {
    if q.fract().abs() < 1e-9 {
        format!("{:.0}", q)
    } else {
        format!("{:.2}", q)
    }
}

/// One fully-resolved holdings row (mark + P/L computed by the caller).
struct HoldingRow {
    symbol: String,
    long: bool,
    qty: f64,
    avg: f64,
    mark: f64,
    pnl: f64,
}

/// One fully-resolved order-history row.
struct OrderRow {
    symbol: String,
    buy: bool,
    qty: f64,
    price: f64,
}

// ── App ───────────────────────────────────────────────────────────────

fn app() -> impl gpui::IntoElement {
    apply_theme(true);

    let (market, set_market) = create_store(MarketState::new());
    let (selected, set_selected) = create_signal(String::from("AAPL"));
    let (window_len, set_window_len) = create_signal(160usize);
    let (qty, set_qty) = create_signal(String::from("100"));
    let (query, set_query) = create_signal(String::new());
    let (live, set_live) = create_signal(true);
    let (holdings_tab, set_holdings_tab) = create_signal(0u8); // 0 = positions, 1 = orders
    // `Toast` is not `PartialEq`, so the stack lives in a Clone-only store.
    let (toasts, set_toasts) = create_store(Vec::<Toast>::new());
    let (toast_id, set_toast_id) = create_signal(1u64);

    // 1.2s market tick; a zero delay pauses the feed.
    let live_ticker = live.clone();
    let delay = create_memo(move || if live_ticker.get() { 1200u64 } else { 0u64 });
    use_interval(
        {
            let set_market = set_market.clone();
            move |cx| set_market.update(cx, |s| s.tick())
        },
        delay,
    );

    let state = market.get();
    let sym = selected.get();
    let inst: Instrument = state
        .instrument(&sym)
        .cloned()
        .unwrap_or_else(|| state.instruments[0].clone());
    let q = query.get().to_lowercase();
    let watch: Vec<Instrument> = state
        .instruments
        .iter()
        .filter(|i| {
            q.is_empty()
                || i.symbol.to_lowercase().contains(&q)
                || i.name.to_lowercase().contains(&q)
        })
        .cloned()
        .collect();

    let cash = state.cash;
    let equity = state.equity();
    let total_pnl = state.total_pnl();
    let holdings: Vec<HoldingRow> = state
        .positions
        .iter()
        .map(|p| {
            let mark = state.instrument(&p.symbol).map(|i| i.price).unwrap_or(p.avg_price);
            let pnl = match p.side {
                Side::Long => (mark - p.avg_price) * p.qty + p.realized,
                Side::Short => (p.avg_price - mark) * p.qty + p.realized,
            };
            HoldingRow {
                symbol: p.symbol.clone(),
                long: p.side == Side::Long,
                qty: p.qty,
                avg: p.avg_price,
                mark,
                pnl,
            }
        })
        .collect();
    let position_count = holdings.len();
    let order_rows: Vec<OrderRow> = state
        .orders
        .iter()
        .take(30)
        .map(|o| OrderRow {
            symbol: o.symbol.clone(),
            buy: o.side == OrderSide::Buy,
            qty: o.qty,
            price: o.price,
        })
        .collect();
    let order_count = state.orders.len();
    let holdings_tabs = [
        (0u8, format!("Positions ({})", position_count)),
        (1u8, format!("Orders ({})", order_count)),
    ];
    let win_now = window_len.get();
    let live_now = live.get();
    let qty_now = qty.get();
    let change = inst.change_pct();
    let change_amt = inst.price - inst.prev_close;

    let push_toast = {
        let set_toasts = set_toasts.clone();
        let toast_id = toast_id.clone();
        let set_toast_id = set_toast_id.clone();
        move |title: String, desc: String, destructive: bool, cx: &mut App| {
            let id = toast_id.get();
            set_toast_id.set(cx, id + 1);
            set_toasts.update(cx, |list| {
                let mut t = Toast::new(id, title).description(desc);
                if destructive {
                    t = t.destructive();
                }
                list.push(t);
            });
        }
    };

    // Factory that builds an order-submit handler per side. It owns cloned
    // handles and clones them again per invocation, so both Buy and Sell can
    // call it.
    let trade = {
        let set_market = set_market.clone();
        let selected = selected.clone();
        let qty = qty.clone();
        let set_qty = set_qty.clone();
        let push = push_toast.clone();
        move |side: OrderSide| {
            let set_market = set_market.clone();
            let selected = selected.clone();
            let qty = qty.clone();
            let set_qty = set_qty.clone();
            let push = push.clone();
            click(move |cx| {
                let sym = selected.get();
                let amount: f64 = qty.get().trim().parse().unwrap_or(0.0);
                if amount <= 0.0 {
                    push(
                        String::from("Invalid quantity"),
                        String::from("Enter a quantity greater than zero."),
                        true,
                        cx,
                    );
                    return;
                }
                let price = set_market.update(cx, |s| {
                    let price = s.instrument(&sym).map(|i| i.price);
                    if s.place_order(&sym, side, amount) {
                        price
                    } else {
                        None
                    }
                });
                if let Some(price) = price {
                    push(
                        format!("{} {} filled", side.label(), sym),
                        format!("{} @ {}", fmt_qty(amount), money(price)),
                        false,
                        cx,
                    );
                    set_qty.set(cx, String::new());
                }
            })
        }
    };
    let set_qty_ticket = set_qty.clone();

    view! {
        <div class="w-full h-full flex flex-col overflow-hidden" style={css! {
            background: "#0b0e11";
            color: #e6e9f2;
        }}>
            // ── Top bar ───────────────────────────────────────────────
            <div class="flex flex-row items-center justify-between px-4" style={css! {
                height: 52px;
                background: #13151c;
                border-bottom-width: 1px;
                border-style: solid;
                border-color: #262936;
            }}>
                <div class="flex flex-row items-center gap-3">
                    <div class="flex items-center justify-center" style={css! {
                        width: 30px; height: 30px; border-radius: 8px;
                        background: #ff7a45; color: "#0b0e11"; font-weight: 800;
                    }}>{"L"}</div>
                    <span class="font-semibold" style={css! { font-size: 15px; }}>{"Longbridge Pro"}</span>
                    <Tag variant={TagVariant::Outline} label={String::from("Simulated")} closable={false} on_close={move |_cx| {}} />
                </div>
                <div class="flex flex-row items-center gap-4">
                    <div class="flex flex-row items-center gap-2">
                        <span class="text-xs" style={css! { color: #7c8295; }}>{"Live feed"}</span>
                        <button
                            style={if live_now {
                                css! {
                                    width: 40px; height: 22px; border-radius: 999px; border-width: 0px;
                                    padding: 2px; display: flex; cursor: pointer;
                                    background: "#ff7a45";
                                }
                            } else {
                                css! {
                                    width: 40px; height: 22px; border-radius: 999px; border-width: 0px;
                                    padding: 2px; display: flex; cursor: pointer;
                                    background: #3a3f4d;
                                }
                            }}
                            hover={css! { opacity: 0.85; }}
                            on:click={click(move |cx| set_live.update(cx, |v| *v = !*v))}
                        >
                            <div style={if live_now {
                                css! { width: 18px; height: 18px; border-radius: 999px; background: #ffffff; margin-left: 18px; }
                            } else {
                                css! { width: 18px; height: 18px; border-radius: 999px; background: #c2c7d4; margin-left: 0px; }
                            }} />
                        </button>
                    </div>
                    <Separator vertical={true} />
                    <div class="flex flex-col items-end">
                        <span class="text-xs" style={css! { color: #7c8295; }}>{"Equity"}</span>
                        <span class="font-mono font-semibold" style={css! { font-size: 13px; }}>{format!("${}", money(equity))}</span>
                    </div>
                    <Avatar initials={String::from("JP")} size={30.0} />
                </div>
            </div>

            <div class="flex flex-row flex-1 overflow-hidden">
                // ── Nav rail ──────────────────────────────────────────
                <div class="flex flex-col items-center pt-3 gap-1" style={css! {
                    width: 72px; background: #13151c;
                    border-right-width: 1px; border-style: solid; border-color: #262936;
                }}>
                    {for_each(["行情", "交易", "资产", "历史", "设置"], move |label: &str, _| {
                        let active = label == "行情";
                        let glyph = match label {
                            "行情" => "▦",
                            "交易" => "⇄",
                            "资产" => "◈",
                            "历史" => "◷",
                            _ => "⚙",
                        };
                        let tint = if active { BRAND } else { "#7c8295" };
                        view! {
                            <div class="flex flex-col items-center justify-center cursor-pointer"
                                style={css! { width: 60px; height: 52px; gap: 3px; border-radius: 10px; }}
                                hover={css! { background: #181b25; }}
                            >
                                {tinted(glyph.to_string(), tint)}
                                <span style={css! { font-size: 10px; color: #7c8295; }}>{label}</span>
                            </div>
                        }
                    })}
                </div>

                // ── Watchlist ─────────────────────────────────────────
                <div class="flex flex-col" style={css! {
                    width: 264px; background: #13151c;
                    border-right-width: 1px; border-style: solid; border-color: #262936;
                }}>
                    <div class="flex flex-col gap-2 p-3">
                        <span class="font-semibold" style={css! { font-size: 13px; }}>{"Watchlist"}</span>
                        <TextField
                            value={query.get()}
                            placeholder="Search symbol"
                            on_input={move |v, cx| set_query.set(cx, v)}
                            class={Some(String::from("w-full"))}
                        />
                    </div>
                    <div class="flex-1 overflow-y-auto flex flex-col">
                        {for_each(watch, {
                            let selected = selected.clone();
                            let set_selected = set_selected.clone();
                            move |i: Instrument, _| {
                                let is_sel = selected.get() == i.symbol;
                                let set_selected = set_selected.clone();
                                let sym = i.symbol.to_string();
                                let chg = i.change_pct();
                                view! {
                                    <div
                                        class="flex flex-row items-center justify-between px-3 cursor-pointer"
                                        style={if is_sel {
                                            css! { height: 54px; background: #1d212c; }
                                        } else {
                                            css! { height: 54px; background: #13151c; }
                                        }}
                                        hover={css! { background: #181b25; }}
                                        on:click={click(move |cx| set_selected.set(cx, sym.clone()))}
                                    >
                                        <div class="flex flex-col" style={css! { gap: 2px; }}>
                                            <span class="font-semibold" style={css! { font-size: 13px; }}>{i.symbol}</span>
                                            <span class="truncate" style={css! { font-size: 11px; color: #7c8295; max-width: 130px; }}>{i.name}</span>
                                        </div>
                                        <div class="flex flex-col items-end" style={css! { gap: 2px; }}>
                                            <span class="font-mono" style={css! { font-size: 13px; }}>{money(i.price)}</span>
                                            {tinted(pct(chg), change_color(chg))}
                                        </div>
                                    </div>
                                }
                            }
                        })}
                    </div>
                </div>

                // ── Center: chart + holdings ─────────────────────────
                <div class="flex-1 flex flex-col gap-3 p-4 overflow-y-auto">
                    // Symbol header
                    <div class="flex flex-row items-end justify-between">
                        <div class="flex flex-col gap-1">
                            <div class="flex flex-row items-center gap-2">
                                <span class="font-semibold" style={css! { font-size: 20px; }}>{inst.symbol}</span>
                                <Tag variant={TagVariant::Secondary} label={String::from(inst.sector)} closable={false} on_close={move |_cx| {}} />
                            </div>
                            <span style={css! { font-size: 12px; color: #7c8295; }}>{inst.name}</span>
                        </div>
                        <div class="flex flex-row items-center gap-3">
                            <div class="flex flex-col items-end gap-1">
                                {tinted(money(inst.price), change_color(change))}
                                <span class="font-mono" style={css! { font-size: 12px; }}>
                                    {tinted(format!("{} {}", signed_money(change_amt), pct(change)), change_color(change))}
                                </span>
                            </div>
                            {chart::change_chip(change)}
                        </div>
                    </div>

                    // Stat strip
                    <div class="flex flex-row gap-2">
                        {stat_block("Day High", money(inst.day_high))}
                        {stat_block("Day Low", money(inst.day_low))}
                        {stat_block("Volume", volume(inst.volume))}
                        {stat_block("Mkt Cap", compact_usd(inst.market_cap))}
                    </div>

                    // Timeframe tabs + chart
                    <Card>
                        <Tabs>
                            <TabsList>
                                {for_each([(40usize, "1D"), (80usize, "1W"), (160usize, "ALL")], move |(w, label), _| {
                                    let set_window_len = set_window_len.clone();
                                    view! {
                                        <TabsTrigger active={win_now == w} on:click={move |cx| set_window_len.set(cx, w)}>
                                            {label}
                                        </TabsTrigger>
                                    }
                                })}
                            </TabsList>
                        </Tabs>
                        {chart::price_chart(&inst, win_now)}
                    </Card>
                    // Positions / Orders
                    <Card>
                        <Tabs>
                            <TabsList>
                                {for_each(holdings_tabs, {
                                    let set_holdings_tab = set_holdings_tab.clone();
                                    let holdings_tab = holdings_tab.clone();
                                    move |(idx, label), _| {
                                        let set_holdings_tab = set_holdings_tab.clone();
                                        view! {
                                            <TabsTrigger active={holdings_tab.get() == idx} on:click={move |cx| set_holdings_tab.set(cx, idx)}>
                                                {label}
                                            </TabsTrigger>
                                        }
                                    }
                                })}
                            </TabsList>
                        </Tabs>
                        {if holdings_tab.get() == 0 {
                            holdings_table(holdings).into_any_element()
                        } else {
                            orders_table(order_rows).into_any_element()
                        }}
                    </Card>
                </div>

                // ── Right: order ticket ───────────────────────────────
                <div class="flex flex-col gap-3 p-4 overflow-y-auto" style={css! {
                    width: 288px; background: #13151c;
                    border-left-width: 1px; border-style: solid; border-color: #262936;
                }}>
                    <Card>
                        <CardHeader title={String::from("Trade")} description={format!("{} · Market order", inst.symbol)} />

                        <span class="text-xs" style={css! { color: #7c8295; }}>{"Quantity"}</span>
                        <TextField
                            value={qty_now.clone()}
                            placeholder="0"
                            on_input={move |v, cx| set_qty_ticket.set(cx, v)}
                            class={Some(String::from("w-full"))}
                        />
                        <div class="flex flex-row gap-2">
                            {for_each(["10", "50", "100", "1K"], move |chip: &str, _| {
                                let set_qty_chip = set_qty.clone();
                                let val = if chip == "1K" { String::from("1000") } else { chip.to_string() };
                                view! {
                                    <Button variant={ButtonVariant::Outline} size={ButtonSize::Sm} on:click={move |cx| set_qty_chip.set(cx, val.clone())}>
                                        {chip}
                                    </Button>
                                }
                            })}
                        </div>

                        <div class="flex flex-row justify-between text-xs" style={css! { color: #7c8295; }}>
                            <span>{"Market price"}</span>
                            <span class="font-mono" style={css! { color: #e6e9f2; }}>{money(inst.price)}</span>
                        </div>
                        <div class="flex flex-row justify-between text-xs" style={css! { color: #7c8295; }}>
                            <span>{"Est. cost"}</span>
                            <span class="font-mono" style={css! { color: #e6e9f2; }}>
                                {format!("${}", money(qty_now.trim().parse::<f64>().unwrap_or(0.0) * inst.price))}
                            </span>
                        </div>
                        <div class="flex flex-row gap-2">
                            <button
                                class="flex-1"
                                style={css! {
                                    height: 40px; border-radius: 8px; border-width: 0px;
                                    background: "#2ebd85"; color: #ffffff; font-weight: 700; cursor: pointer;
                                }}
                                hover={css! { opacity: 0.88; }}
                                on:click={trade(OrderSide::Buy)}
                            >{"Buy / Long"}</button>
                            <button
                                class="flex-1"
                                style={css! {
                                    height: 40px; border-radius: 8px; border-width: 0px;
                                    background: "#f23645"; color: #ffffff; font-weight: 700; cursor: pointer;
                                }}
                                hover={css! { opacity: 0.88; }}
                                on:click={trade(OrderSide::Sell)}
                            >{"Sell / Short"}</button>
                        </div>
                    </Card>

                    <Card>
                        <CardHeader title={String::from("Account")} description={String::from("Paper trading")} />
                        {account_row("Cash", format!("${}", money(cash)))}
                        {account_row("Equity", format!("${}", money(equity)))}
                        <div class="flex flex-row justify-between items-center">
                            <span class="text-xs" style={css! { color: #7c8295; }}>{"Total P/L"}</span>
                            {tinted(signed_money(total_pnl), change_color(total_pnl))}
                        </div>
                    </Card>
                </div>
            </div>

            <ToastViewport
                toasts={toasts.get()}
                on_dismiss={move |id: u64, cx| set_toasts.update(cx, |list| list.retain(|t| t.id != id))}
            />
        </div>
    }
}

fn stat_block(label: &'static str, value: String) -> gpui::AnyElement {
    view! {
        <div class="flex-1 flex flex-col gap-1 px-3 py-2" style={css! {
            background: #13151c;
            border-width: 1px; border-style: solid; border-color: #262936;
            border-radius: 8px;
        }}>
            <span style={css! { font-size: 11px; color: #7c8295; }}>{label}</span>
            <span class="font-mono font-semibold" style={css! { font-size: 14px; color: #e6e9f2; }}>{value}</span>
        </div>
    }
    .into_any_element()
}

fn account_row(label: &'static str, value: String) -> gpui::AnyElement {
    view! {
        <div class="flex flex-row justify-between items-center">
            <span class="text-xs" style={css! { color: #7c8295; }}>{label}</span>
            <span class="font-mono" style={css! { font-size: 13px; color: #e6e9f2; }}>{value}</span>
        </div>
    }
    .into_any_element()
}

fn table_header(text: &'static str) -> gpui::AnyElement {
    view! {
        <span style={css! { font-size: 11px; color: #7c8295; }}>{text}</span>
    }
    .into_any_element()
}

fn holdings_table(rows: Vec<HoldingRow>) -> gpui::AnyElement {
    if rows.is_empty() {
        return view! {
            <div class="flex items-center justify-center" style={css! { height: 96px; color: #7c8295; font-size: 13px; }}>
                {"No open positions — place an order to get started."}
            </div>
        }
        .into_any_element();
    }

    view! {
        <div class="flex flex-col">
            <div class="flex flex-row py-2" style={css! { border-bottom-width: 1px; border-style: solid; border-color: #262936; }}>
                <div class="flex-1">{table_header("Symbol")}</div>
                <div style={css! { width: 70px; }}>{table_header("Side")}</div>
                <div class="font-mono text-right" style={css! { width: 80px; }}>{table_header("Qty")}</div>
                <div class="font-mono text-right" style={css! { width: 90px; }}>{table_header("Avg Price")}</div>
                <div class="font-mono text-right" style={css! { width: 90px; }}>{table_header("Mark")}</div>
                <div class="font-mono text-right" style={css! { width: 100px; }}>{table_header("P/L")}</div>
            </div>
            {for_each(rows, move |r: HoldingRow, _| {
                let side_label = if r.long { "LONG" } else { "SHORT" };
                let side_tint = if r.long { UP } else { DOWN };
                let pnl_tint = change_color(r.pnl);
                view! {
                    <div class="flex flex-row items-center py-2" style={css! {
                        border-bottom-width: 1px; border-style: solid; border-color: #262936;
                    }}>
                        <div class="flex-1 font-semibold" style={css! { font-size: 13px; }}>{r.symbol}</div>
                        <div style={css! { width: 70px; }}>
                            {tinted(side_label.to_string(), side_tint)}
                        </div>
                        <div class="font-mono text-right" style={css! { width: 80px; font-size: 13px; }}>{fmt_qty(r.qty)}</div>
                        <div class="font-mono text-right" style={css! { width: 90px; font-size: 13px; }}>{money(r.avg)}</div>
                        <div class="font-mono text-right" style={css! { width: 90px; font-size: 13px; }}>{money(r.mark)}</div>
                        <div class="font-mono text-right font-semibold" style={css! { width: 100px; font-size: 13px; }}>
                            {tinted(signed_money(r.pnl), pnl_tint)}
                        </div>
                    </div>
                }
            })}
        </div>
    }
    .into_any_element()
}

fn orders_table(rows: Vec<OrderRow>) -> gpui::AnyElement {
    if rows.is_empty() {
        return view! {
            <div class="flex items-center justify-center" style={css! { height: 96px; color: #7c8295; font-size: 13px; }}>
                {"No orders yet — executed trades appear here."}
            </div>
        }
        .into_any_element();
    }

    view! {
        <div class="flex flex-col">
            <div class="flex flex-row items-center py-2" style={css! { border-bottom-width: 1px; border-style: solid; border-color: #262936; }}>
                <div class="flex-1">{table_header("Symbol")}</div>
                <div style={css! { width: 70px; }}>{table_header("Side")}</div>
                <div class="font-mono text-right" style={css! { width: 80px; }}>{table_header("Qty")}</div>
                <div class="font-mono text-right" style={css! { width: 100px; }}>{table_header("Fill Price")}</div>
                <div style={css! { width: 90px; }}>{table_header("Status")}</div>
            </div>
            {for_each(rows, move |r: OrderRow, _| {
                let side_label = if r.buy { "BUY" } else { "SELL" };
                let side_tint = if r.buy { DOWN } else { UP };
                view! {
                    <div class="flex flex-row items-center py-2" style={css! {
                        border-bottom-width: 1px; border-style: solid; border-color: #262936;
                    }}>
                        <div class="flex-1 font-semibold" style={css! { font-size: 13px; }}>{r.symbol}</div>
                        <div style={css! { width: 70px; }}>
                            {tinted(side_label.to_string(), side_tint)}
                        </div>
                        <div class="font-mono text-right" style={css! { width: 80px; font-size: 13px; }}>{fmt_qty(r.qty)}</div>
                        <div class="font-mono text-right" style={css! { width: 100px; font-size: 13px; }}>{money(r.price)}</div>
                        <div style={css! { width: 90px; }}>
                            <span style={css! { font-size: 11px; font-weight: 600; color: "#2ebd85"; }}>{"Filled"}</span>
                        </div>
                    </div>
                }
            })}
        </div>
    }
    .into_any_element()
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();
    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(1500.), px(940.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| vgui::mount(window, cx, app),
        )
        .unwrap();
    };

    #[cfg(not(target_family = "wasm"))]
    gpui_app.run(launch);
    #[cfg(target_family = "wasm")]
    std::mem::forget(gpui_app.run_embedded(launch));
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    vgui::intercept_keyboard_events();
    run();
}
