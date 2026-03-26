use crate::chatter::chatter_state::Conversation;
use web_sys::HtmlInputElement;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MessageInputProps {
    pub on_send_message: Callback<(Conversation, String)>,
    pub conversation: Conversation,
}

#[function_component(MessageInput)]
pub fn message_input(props: &MessageInputProps) -> Html {
    let input_value = use_state(String::new);
    let input_ref = use_node_ref();

    let on_input_change = {
        let input_value = input_value.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            input_value.set(input.value());
        })
    };

    let send_message = {
        let input_value = input_value.clone();
        let input_ref = input_ref.clone();
        let on_send = props.on_send_message.clone();
        let conversation = props.conversation.clone();

        Callback::from(move |_| {
            let text = (*input_value).clone();
            if !text.trim().is_empty() {
                on_send.emit((conversation.clone(), text));
                input_value.set(String::new());

                if let Some(input) = input_ref.cast::<HtmlInputElement>() {
                    input.set_value("");
                }
            }
        })
    };

    let on_send_click = {
        let send_message = send_message.clone();
        Callback::from(move |_: MouseEvent| {
            send_message.emit(());
        })
    };

    let on_key_press = {
        let send_message = send_message.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" && !e.shift_key() {
                e.prevent_default();
                send_message.emit(());
            }
        })
    };

    html! {
        <div class="bg-white border-t border-gray-200 px-4 py-3">
            <div class="flex items-center gap-2">
                <input
                    ref={input_ref}
                    type="text"
                    value={(*input_value).clone()}
                    oninput={on_input_change}
                    onkeypress={on_key_press}
                    placeholder="Type a message..."
                    class="flex-1 px-4 py-3 bg-gray-100 rounded-full focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                />
                <button
                    onclick={on_send_click}
                    disabled={input_value.trim().is_empty()}
                    class={classes!(
                        "p-3", "rounded-full", "transition",
                        if !input_value.trim().is_empty() {
                            "bg-blue-500 hover:bg-blue-600 text-white"
                        } else {
                            "bg-gray-200 text-gray-400 cursor-not-allowed"
                        }
                    )}
                >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
                    </svg>
                </button>
            </div>
        </div>
    }
}
