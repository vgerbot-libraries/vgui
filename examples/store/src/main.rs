#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

// ---------------------------------------------------------------------------
// State — a single aggregate struct held in a `create_store`.
//
// `CartState` does NOT derive `PartialEq`. The store always notifies on write;
// fine-grained filtering is delegated to `Store::select`, whose slice type
// `U` must implement `PartialEq`.
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Product {
    id: u32,
    name: &'static str,
    price: f64,
}

#[derive(Clone)]
struct CartItem {
    product: Product,
    qty: u32,
}

#[derive(Clone)]
struct CartState {
    items: Vec<CartItem>,
    discount_pct: f64, // 0.0 – 100.0
    tax_pct: f64,      // 0.0 – 100.0
}

impl CartState {
    fn new() -> Self {
        Self {
            items: vec![
                CartItem {
                    product: Product { id: 0, name: "Rust Book", price: 39.99 },
                    qty: 1,
                },
                CartItem {
                    product: Product { id: 1, name: "Mechanical Keyboard", price: 129.00 },
                    qty: 1,
                },
                CartItem {
                    product: Product { id: 2, name: "Coffee Mug", price: 12.50 },
                    qty: 2,
                },
            ],
            discount_pct: 0.0,
            tax_pct: 8.0,
        }
    }

    fn subtotal(&self) -> f64 {
        self.items.iter().map(|i| i.product.price * i.qty as f64).sum()
    }
}

// ---------------------------------------------------------------------------
// UI helpers
// ---------------------------------------------------------------------------

fn money(v: f64) -> String {
    format!("${:.2}", v)
}

fn cart_row(
    name: &str,
    price: f64,
    qty: u32,
    on_inc: impl Fn(&mut gpui::App) + 'static,
    on_dec: impl Fn(&mut gpui::App) + 'static,
    on_remove: impl Fn(&mut gpui::App) + 'static,
) -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-row items-center gap-3 p-2 rounded bg-[#2a2a3a]">
            <span class="flex-1 text-sm text-white">{name.to_string()}</span>
            <span class="text-sm text-[#aaa] w-20 text-right">{money(price)}</span>
            <div class="flex flex-row items-center gap-1">
                <button
                    class="w-6 h-6 rounded bg-[#444] text-white text-sm hover:bg-[#555]"
                    on:click={click(on_dec)}
                >
                    {"-"}
                </button>
                <span class="w-8 text-center text-white text-sm">{qty.to_string()}</span>
                <button
                    class="w-6 h-6 rounded bg-[#444] text-white text-sm hover:bg-[#555]"
                    on:click={click(on_inc)}
                >
                    {"+"}
                </button>
            </div>
            <button
                class="px-2 py-1 rounded bg-[#dc2626] text-white text-xs hover:bg-[#b91c1c]"
                on:click={click(on_remove)}
            >
                {"Remove"}
            </button>
        </div>
    }
}

fn slider_row(
    label: &str,
    value: f64,
    on_change: impl Fn(f64, &mut gpui::App) + 'static,
) -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-row items-center gap-3">
            <span class="text-sm text-[#ccc] w-24">{label.to_string()}</span>
            <input
                type="range"
                class="flex-1"
                min={0.0f64}
                max={100.0f64}
                step={1.0f64}
                value={value}
                on:change={on_change}
            />
            <span class="text-sm text-white w-12 text-right">{format!("{:.0}%", value)}</span>
        </div>
    }
}

fn summary_row(label: &str, value: &str, highlight: bool) -> impl gpui::IntoElement {
    let class = if highlight {
        "flex flex-row justify-between text-sm font-bold text-white border-t border-[#444] pt-2 mt-2"
    } else {
        "flex flex-row justify-between text-sm text-[#ccc]"
    };
    view! {
        <div class={class}>
            <span>{label.to_string()}</span>
            <span>{value.to_string()}</span>
        </div>
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

fn app() -> impl gpui::IntoElement {
    let (cart, set_cart) = create_store(CartState::new());

    // Fine-grained selectors — each derives a ReadSignal from a single slice.
    // Updating `discount_pct` does NOT cause `item_count` or `subtotal` to
    // re-render; updating a quantity does NOT re-render `discount_pct`.
    let item_count = cart.select(|s| s.items.len());
    let subtotal = cart.select(|s| s.subtotal());
    let discount_pct = cart.select(|s| s.discount_pct);
    let tax_pct = cart.select(|s| s.tax_pct);

    // Derived values built on top of selectors — these are memos that depend
    // on multiple selector signals. They recompute only when their inputs
    // change, thanks to the fine-grained selectors below them.
    let discount_amount = create_memo({
        let subtotal = subtotal.clone();
        let discount_pct = discount_pct.clone();
        move || subtotal.get() * discount_pct.get() / 100.0
    });

    let after_discount = create_memo({
        let subtotal = subtotal.clone();
        let discount_amount = discount_amount.clone();
        move || subtotal.get() - discount_amount.get()
    });

    let tax_amount = create_memo({
        let after_discount = after_discount.clone();
        let tax_pct = tax_pct.clone();
        move || after_discount.get() * tax_pct.get() / 100.0
    });

    let total = create_memo({
        let after_discount = after_discount.clone();
        let tax_amount = tax_amount.clone();
        move || after_discount.get() + tax_amount.get()
    });

    // Read current values for this render.
    let items = cart.with(|s| s.items.clone());
    let cur_discount = discount_pct.get();
    let cur_tax = tax_pct.get();
    let cur_subtotal = subtotal.get();
    let cur_discount_amt = discount_amount.get();
    let cur_after = after_discount.get();
    let cur_tax_amt = tax_amount.get();
    let cur_total = total.get();
    let cur_count = item_count.get();

    let set_cart_discount = set_cart.clone();
    let set_cart_tax = set_cart.clone();
    let set_cart_clear = set_cart.clone();

    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#1a1a2e] w-full h-full text-white overflow-hidden">
            <span class="text-xl font-bold text-center">{"Shopping Cart (create_store)"}</span>

            // Item count — driven by `item_count` selector only.
            <span class="text-sm text-[#aaa] text-center">
                {format!("{} items in cart", cur_count)}
            </span>

            // Cart items
            <div class="flex flex-col gap-2 flex-1 overflow-y-auto">
                <For each={items.clone()} fallback={view! {
                    <div class="text-center text-[#888] py-8 text-sm">
                        {"Cart is empty."}
                    </div>
                }}>
                    {move |item: CartItem, _i: usize| {
                        let id = item.product.id;
                        let name = item.product.name;
                        let price = item.product.price;
                        let qty = item.qty;
                        let sc = set_cart.clone();
                        let sc2 = set_cart.clone();
                        let sc3 = set_cart.clone();
                        cart_row(
                            name,
                            price,
                            qty,
                            move |cx| sc.update(cx, |s| {
                                if let Some(it) = s.items.iter_mut().find(|i| i.product.id == id) {
                                    it.qty += 1;
                                }
                            }),
                            move |cx| sc2.update(cx, |s| {
                                if let Some(it) = s.items.iter_mut().find(|i| i.product.id == id) {
                                    if it.qty > 1 { it.qty -= 1; }
                                }
                            }),
                            move |cx| sc3.update(cx, |s| {
                                s.items.retain(|i| i.product.id != id);
                            }),
                        )
                    }}
                </For>
            </div>

            // Sliders for discount and tax — each updates one field of the
            // store. Only the affected selectors and their dependents
            // recompute; the item list is unaffected.
            <div class="flex flex-col gap-2 p-3 rounded bg-[#252535]">
                <span class="text-sm font-bold text-[#ccc]">{"Pricing Controls"}</span>
                {slider_row("Discount", cur_discount, move |v, cx| {
                    set_cart_discount.update(cx, |s| s.discount_pct = v);
                })}
                {slider_row("Tax", cur_tax, move |v, cx| {
                    set_cart_tax.update(cx, |s| s.tax_pct = v);
                })}
            </div>

            // Order summary — each line reads a different selector / memo.
            // Changing the discount only re-runs `discount_amount`,
            // `after_discount`, `tax_amount`, and `total` — not `item_count`
            // or the item list itself.
            <div class="flex flex-col gap-1 p-3 rounded bg-[#252535]">
                <span class="text-sm font-bold text-[#ccc] mb-1">{"Order Summary"}</span>
                {summary_row("Subtotal", &money(cur_subtotal), false)}
                {summary_row("Discount", &format!("-{}", money(cur_discount_amt)), false)}
                {summary_row("After Discount", &money(cur_after), false)}
                {summary_row("Tax", &format!("+{}", money(cur_tax_amt)), false)}
                {summary_row("Total", &money(cur_total), true)}
            </div>

            <button
                class="p-2 rounded bg-[#dc2626] text-white text-sm hover:bg-[#b91c1c]"
                on:click={click(move |cx| {
                    set_cart_clear.update(cx, |s| s.items.clear());
                })}
            >
                {"Clear Cart"}
            </button>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(520.), px(700.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| vgui::mount(cx, app),
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
