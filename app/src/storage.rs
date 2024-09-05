use codee::string::FromToStringCodec;
use leptos::*;
use leptos_use::storage::use_local_storage;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::js_sys::JsString;

async fn upload_file(file: web_sys::File) {
    let name = file.name();
    let text = JsFuture::from(file.text())
        .await
        .unwrap()
        .unchecked_into::<JsString>();

    let (files, set_files, _) = use_local_storage::<String, FromToStringCodec>("files");
    let (_, set_content, _) = use_local_storage::<String, FromToStringCodec>(format!("f-{name}"));

    let files = files.get_untracked();
    let mut files: Vec<_> = files.lines().collect();
    if !files.contains(&name.as_str()) {
        files.push(&name);
    }

    set_files.set(files.join("\n"));
    set_content.set(format!("{text}"));
    // TODO do we need to manually dispose here?
}

pub fn get_file(name: &str) -> Option<String> {
    // no reactive tracking for now
    let s = window().local_storage().unwrap().unwrap();
    s.get_item(&format!("f-{name}")).unwrap()
}

pub fn get_file_list() -> Signal<Vec<String>> {
    let (files, _, _) = use_local_storage::<String, FromToStringCodec>("files");
    (move || {
        files
            .get()
            .lines()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    })
    .into()
}

#[component]
pub fn StorageSidebar() -> impl IntoView {
    let file_list = get_file_list();

    let file_input = create_node_ref::<html::Input>();
    view! {
        <div class="sidebar">
            <div>
                <input type="file" node_ref=file_input />
                <button on:click= move |_| {
                        let elem_files = file_input.get().unwrap().files().unwrap();
                        if elem_files.length() > 0 {
                            let file = elem_files.get(0).unwrap();
                            spawn_local(async move {
                                upload_file(file).await;
                                file_input.get_untracked().unwrap().set_value("");
                            });
                        }
                    } >
                    <span class="material-symbols-outlined">upload_file</span>
                </button>
            </div>
            <For
                each=move || file_list.get()
                key= |e| e.clone()
                children= |e| view!{<div> {e} </div>}
            />
        </div>
    }
}
