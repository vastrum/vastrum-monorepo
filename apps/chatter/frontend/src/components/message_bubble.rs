use crate::chatter::chatter_state::ConversationMessage;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MessageBubbleProps {
    pub message: ConversationMessage,
}

#[function_component(MessageBubble)]
pub fn message_bubble(props: &MessageBubbleProps) -> Html {
    let message = &props.message;

    html! {
        <div
            class={classes!(
                "flex",
                if message.from_me { "justify-end" } else { "justify-start" }
            )}
        >
            <div
                class={classes!(
                    "max-w-xs", "lg:max-w-md", "xl:max-w-lg",
                    if message.from_me {
                        ""
                    } else {
                        "flex flex-col"
                    }
                )}
            >
                // Show author name for group chat messages (non-sent messages)
                /*
                {
                    if !message.sent && message.author.is_some() {
                        html! {
                            <span class="text-xs font-semibold text-gray-600 mb-1 ml-1">
                                {message.author.as_ref().unwrap()}
                            </span>
                        }
                    } else {
                        html! {}
                    }
                }
                 */
                <div
                    class={classes!(
                        "px-4", "py-2", "rounded-2xl", "shadow-sm",
                        if message.from_me {
                            "bg-blue-500 text-white rounded-br-sm"
                        } else {
                            "bg-white text-gray-800 rounded-bl-sm"
                        }
                    )}
                >
                    <p class="text-sm break-words">{&message.content}</p>
                    <p
                        class={classes!(
                            "text-xs", "mt-1",
                            if message.from_me { "text-blue-100" } else { "text-gray-500" }
                        )}
                    >
                        //{&message.time}
                    </p>
                </div>
            </div>
        </div>
    }
}
