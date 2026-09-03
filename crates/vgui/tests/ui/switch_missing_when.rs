use vgui::prelude::*;

fn main() {
    let _ = view! {
        <Switch>
            <Match>{"x"}</Match>
        </Switch>
    };
}
