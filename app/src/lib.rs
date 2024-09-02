#![allow(clippy::needless_lifetimes)]

use codee::string::FromToStringCodec;
use leptos::*;
use leptos_use::signal_debounced;
use leptos_use::storage::use_local_storage;
use web_sys::Event;

mod js_types;

#[component]
pub fn App() -> impl IntoView {
    // for now source code is a string stored in local storage
    let (storage, set_storage, _) = use_local_storage::<String, FromToStringCodec>("testcode");
    let (code, set_code) = create_signal(storage.get_untracked());
    let code_debounced = signal_debounced(code, 300.0);
    create_effect(move |_| {
        set_storage.set(code_debounced.get());
    });

    let output = create_memo(move |_| {
        with!(|code_debounced| {
            let program = match interpreter::grammar::ProgramParser::new().parse(code_debounced) {
                Ok(r) => r,
                Err(e) => return e.to_string(),
            };
            match interpreter::execute(&program) {
                Ok(r) => r,
                Err(e) => format!("{e:?}"),
            }
        })
    });

    view! {
        <title>"Program Demo"</title>
        <div class="background">
            <div> /* logo area */ </div>
            <div class="header">
                <h2>dummy header</h2>
            </div>
            <div class="sidebar">
                sidebar
            </div>
            <div style:display="grid"
                 style:grid-template-columns="minmax(0px,50fr) 12px minmax(0px,50fr)">
                <div class="floating-pane">
                    <code-editor
                        initialSrc=move || code.get()
                        on:myInput=move |ev: Event| {
                            set_code.set(event_target_value(&ev))
                        }
                    />
                </div>
                <div>
                    // TODO: handle bar for resizing
                </div>
                <div class="floating-pane">
                    <div class="output" >
                        { output }
                    </div>
                </div>
            </div>
        </div>
    }
}
