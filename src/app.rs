use leptos::ev::{InputEvent, Event};
use leptos::leptos_dom::console_log;
use leptos::leptos_dom::ev::{SubmitEvent};
use leptos::*;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "tauri"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

//let greet = move |ev: SubmitEvent| {
//    ev.prevent_default();
//    spawn_local(async move {
//        if name.get().is_empty() {
//            return;
//        }
//
//        let args = to_value(&GreetArgs { name: &name.get() }).unwrap();
//        // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
//        let new_msg = invoke("greet", args).await.as_string().unwrap();
//        set_greet_msg.set(new_msg);
//    });
//};

//#[derive(Serialize, Deserialize)]
//struct GreetArgs<'a> {
//    name: &'a str,
//}

#[derive(Serialize, Deserialize)]
struct LogArgs<'a> {
    msg: &'a str
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (prompt, set_prompt) = create_signal(cx, String::new());


    let prompt_change = move |e: Event| {
        set_prompt.set(event_target_value(&e));
        console_log(&format!("{}", prompt.get()));
    };

    view! { cx,
        <main class="container">
            <div class="chatlog">
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
                <h1>"hi"</h1>
            </div>
            <div class="prompt-area">
                <textarea value={ move || prompt.get() } on:input=prompt_change />
            </div>
        </main>
    }
}
