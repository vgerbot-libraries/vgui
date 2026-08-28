use vgui::prelude::*;

fn main() {
    let _ = view! {
        <input type="text">
            <span>{"child"}</span>
        </input>
    };
}
