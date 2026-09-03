use vgui::prelude::*;

fn main() {
    let _ = view! {
        <Index each={vec![1u32, 2, 3]}>
            {"not a closure"}
        </Index>
    };
}
