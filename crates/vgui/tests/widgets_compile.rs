//! Compile/smoke test for button tab, img alt, textarea rows, and meter.

use gpui::{App, Element, IntoElement};
use vgui::prelude::*;
use vgui::view;

struct RenderScope;

impl RenderScope {
    fn new() -> Self {
        vgui::__test_enter_render_scope();
        RenderScope
    }
}

impl Drop for RenderScope {
    fn drop(&mut self) {
        vgui::__test_exit_render_scope();
    }
}

fn assert_into_any<E: IntoElement>(_: fn() -> E) {}

#[test]
fn widgets_compile_and_produce_elements() {
    let _scope = RenderScope::new();

    let button = view! { <button>{"Go"}</button> };
    let _ = button.into_any_element();

    let img = view! { <img src={"x.png"} alt={"logo"} /> };
    let _ = img.into_any_element();

    // `text_area` needs a VguiRoot slot; type-check `rows` without constructing.
    assert_into_any(|| view! { <textarea rows={4u32} /> });

    let meter = view! {
        <meter value={0.3f64} min={0f64} max={1f64} low={0.2f64} high={0.8f64} optimum={0.5f64} />
    };
    let _ = meter.into_any_element();

    assert_into_any(|| view! {
        <form on:submit={move |_cx| {}} on:reset={move |_cx| {}}>
            <input type="text" required={true} pattern={"abc"} minlength={1usize} />
            <input type="submit" value="Go" />
            <input type="reset" value="Clear" />
        </form>
    });

    // datalist + input with list
    let datalist = view! {
        <datalist id="fruits" options={vec!["apple".to_string(), "banana".to_string()]} />
    };
    let _ = datalist.into_any_element();

    assert_into_any(|| view! {
        <input type="text" list="fruits" />
    });

    // select multiple
    assert_into_any(|| view! {
        <select
            multiple={true}
            options={vec![
                ("a".to_string(), "Alpha".to_string()),
                ("b".to_string(), "Beta".to_string()),
            ]}
            value={"a,b".to_string()}
            on:change={move |_v: &str, _cx: &mut App| {}}
        />
    });

    // select with groups
    assert_into_any(|| view! {
        <select
            groups={vec![
                ("Group 1".to_string(), vec![
                    ("1a".to_string(), "Option 1A".to_string()),
                ]),
            ]}
            value={"1a".to_string()}
            on:change={move |_v: &str, _cx: &mut App| {}}
        />
    });

    // output element
    let output = view! { <output>{"42"}</output> };
    let _ = output.into_any_element();

    // option and optgroup as standalone tags (pure div aliases)
    let opt = view! { <option>{"x"}</option> };
    let _ = opt.into_any_element();
    let optgroup = view! { <optgroup>{"group"}</optgroup> };
    let _ = optgroup.into_any_element();

    // input type=color
    assert_into_any(|| view! {
        <input type="color" value={"#ff0000".to_string()} />
    });

    // ARIA: role and aria:* attributes on div
    assert_into_any(|| view! {
        <div role="button" aria:label="Save" aria:expanded={false}>
            {"Save"}
        </div>
    });

    // ARIA: aria:selected, aria:toggled, aria:value
    assert_into_any(|| view! {
        <div role="checkbox" aria:label="Agree" aria:selected={true} aria:toggled="true">
            {"Agree to terms"}
        </div>
    });

    // ARIA: aria:description, aria:keyshortcuts, aria:valuenow
    assert_into_any(|| view! {
        <div role="slider" aria:label="Volume" aria:description="Adjust volume" aria:keyshortcuts="Ctrl+Up" aria:valuenow={50f64}>
            {"Volume"}
        </div>
    });

    // ARIA: role on nav element
    assert_into_any(|| view! {
        <nav role="navigation" aria:label="Main">
            <a href={"#home".to_string()}>{"Home"}</a>
        </nav>
    });

    // ARIA: aria:placeholder
    assert_into_any(|| view! {
        <div role="textbox" aria:label="Search" aria:placeholder="Type here...">
            {"Search"}
        </div>
    });
}
