# Longbridge Pro Trading Terminal Example

## Live Demo

<iframe src="../wasm/longbridge-pro/" width="100%" height="780" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

A Longbridge-styled desktop trading terminal built with `vgui-shadcn` components and a hand-drawn canvas chart. The layout is the classic three-pane broker workstation: a searchable, live-ticking watchlist on the left; a symbol header, stat strip, timeframe-switchable price chart, and Positions/Orders tables in the center; and a market-order ticket plus account summary on the right.

Prices are a deterministic random walk over eight US and Hong Kong instruments. Market orders fill instantly at the live price, cash moves, and opposing positions are netted with realized P/L (a buy closes a short before extending a long, and vice versa). Fills surface as toasts. There is no network or brokerage backend — the whole simulator is in `market.rs`.

## Key Concepts

- **Single brokerage store.** One `create_store(MarketState)` owns instruments, positions, orders, cash, and the PRNG; every read is reactive, and `place_order` mutates cash, order history, and netting in one transaction.
- **Live feed with `use_interval`.** A `create_memo` maps the live toggle to a tick interval (1200ms running, `0` paused); the interval calls `set_market.update(cx, |s| s.tick())`.
- **Canvas chart.** `chart.rs` draws grid, area, line, and a last-price tag to a fixed-size `Context2D`. Timeframe tabs (`1D` / `1W` / `ALL`) only change how many history points are shown.
- **Long/short netting.** `apply_fill` first reduces an opposing position, booking realized P/L, then opens or extends a position in the order's direction.
- **Toast stack in a store.** `Toast` is not `PartialEq`, so the `Vec<Toast>` lives in a Clone-only store rather than a signal; a separate id signal supplies unique toast keys.
- **Runtime colors.** `css!` colors are compile-time, so up/down tints (red up, green down — the Asian-market convention) use a `tinted(text, hex)` helper returning a `gpui::div().text_color(..)`.
- **Module split.** `market.rs` (simulation + formatting), `chart.rs` (canvas rendering), `main.rs` (state wiring and views).

## Running

**Native:**

    cargo run -p vgui-longbridge-pro

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-longbridge-pro --release
    wasm-bindgen --target web --out-dir examples/longbridge-pro/dist \
        --no-typescript target/wasm32-unknown-unknown/release/longbridge-pro.wasm
    python3 scripts/serve_plain.py 8080 examples/longbridge-pro

## Source Code

See `examples/longbridge-pro/src/main.rs`, `src/market.rs`, and `src/chart.rs`.
