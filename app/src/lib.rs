#![allow(clippy::needless_lifetimes)]

use codee::string::FromToStringCodec;
use leptos::*;
use leptos_use::signal_debounced;
use leptos_use::storage::use_local_storage;
use web_sys::Event;

use storage::StorageSidebar;
use svg_graph::SVGGraph;

mod js_types;
mod storage;
mod svg_graph;

struct ExecEnv {}

impl interpreter::execution::Env for ExecEnv {
    fn read_file(&self, name: &str) -> Option<String> {
        storage::get_file(name)
    }
}

#[component]
pub fn App() -> impl IntoView {
    // for now source code is a string stored in local storage
    let (code_storage, set_code_storage, _) =
        use_local_storage::<String, FromToStringCodec>("testcode");
    let (code, set_code) = create_signal(code_storage.get_untracked());
    let code_debounced = signal_debounced(code, 300.0);
    create_effect(move |_| {
        set_code_storage.set(code_debounced.get());
    });

    let output = create_memo(move |_| {
        with!(|code_debounced| {
            let program = interpreter::grammar::ProgramParser::new()
                .parse(code_debounced)
                .map_err(|e| e.to_string())?;
            Ok(interpreter::execution::execute(&program, &ExecEnv {}))
        })
    });

    view! {
        <title>"Control Playground"</title>
        <div class="background">
            <div> /* logo area */ </div>
            <div class="header">
                <h2>Control Playground</h2>
            </div>
            <div class="floating-pane">
                <StorageSidebar />
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
                        <Output output=output />
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Output(
    #[prop(into)] output: Signal<Result<Vec<interpreter::execution::Output>, String>>,
) -> impl IntoView {
    move || match output.get() {
        Err(e) => view! {
            <span class="error" >
                Syntax Error
                <br/>
                { e.to_string() }
            </span>
        }
        .into_view(),
        Ok(output) => output
            .into_iter()
            .map(|el| view! {<OutputElement element=el/>})
            .collect_view(),
    }
}

#[component]
pub fn OutputElement(element: interpreter::execution::Output) -> impl IntoView {
    view! {
        <div class="element" >
            {
                use interpreter::execution::Output::*;
            match element {
                Err(e) => view!{ <span class="error"> { format!("{e:?}") } </span> }.into_view(),
                Text(t) => t.trim_end().to_string().into_view(),
                Plot(data) => view!{ <SVGGraph data={move || data.clone()} initial_height=300.0 /> },
            } }
        </div>
    }
}
