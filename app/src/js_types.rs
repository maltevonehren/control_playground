#![allow(unused_imports)]
#![allow(clippy::all)]
use wasm_bindgen::prelude::*;
use web_sys::*;

#[wasm_bindgen]
extern "C" {

    // not needed for now since we only access `value` which is also supported on other js objects
    // # [wasm_bindgen (extends = HtmlElement , extends = Element , extends = Node , extends = EventTarget , extends =  js_sys::Object , js_name = CodeEditorWC , typescript_type = "CodeEditorWC")]
    // #[derive(Debug, Clone, PartialEq, Eq)]
    // pub type CodeEditorWC;
    // # [wasm_bindgen (structural , method , getter , js_class = "CodeEditorWC" , js_name = value)]
    // pub fn value(this: &CodeEditorWC) -> String;
}
