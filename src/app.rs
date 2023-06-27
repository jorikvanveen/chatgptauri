use leptos::ev::{InputEvent, Event, KeyboardEvent};
use leptos::html::Textarea;
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

#[derive(Serialize, Deserialize, Clone)]
enum Role {
    User,
    Assistant
}

#[derive(Serialize, Deserialize, Clone)]
struct ChatMessage {
    message: String,
    role: Role,
    key: usize
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let (prompt, set_prompt) = create_signal(cx, String::new());
    let (messages, set_messages) = create_signal::<Vec<ChatMessage>>(cx, vec![
        ChatMessage {
            message: "HI".into(),
            role: Role::User,
            key: 0
        },
        ChatMessage {
            message: "HI im a language model or whatever".into(),
            role: Role::Assistant,
            key: 1
        }
    ]);
    let (input_debounce, set_input_debounce) = create_signal(cx, false);

    let prompt_change = move |e: Event| {
        if input_debounce.get() {
            set_prompt.set(String::new());
            set_input_debounce.set(false);
            return
        }

        console_log("Change");
        set_prompt.set(event_target_value(&e));
        console_log(&format!("{}", prompt.get()));
    };

    let keydown = move |e: KeyboardEvent| {
        console_log("Keydown");
        if e.key() == "Enter" && e.ctrl_key() {
            console_log("Submit");
            set_input_debounce.set(true);
            // Submit
            set_prompt.set(String::new());
        }
    };

    view! { cx,
        <main class="container">
            <div class="chatlog">
                <For
                    each=move || messages.get()
                    key=|message| message.key
                    view=move |cx, message: ChatMessage| {
                        view! {
                            cx,
                            <p>{ message.message } </p>
                        }
                    }
                />
            </div>
            <div class="prompt-area">
                <textarea value={ move || prompt.get() } on:keydown=keydown on:input=prompt_change />
            </div>
        </main>
    }
}
