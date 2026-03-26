use crate::chatter::chatter_state::ConversationMessage;
use crate::components::message_bubble::MessageBubble;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MessageListProps {
    pub messages: Option<Vec<ConversationMessage>>,
}

#[function_component(MessageList)]
pub fn message_list(props: &MessageListProps) -> Html {
    let messages_end_ref = use_node_ref();

    // Scroll to bottom when messages change
    {
        let messages_end_ref = messages_end_ref.clone();
        let messages = props.messages.clone();
        use_effect_with(messages, move |_| {
            if let Some(element) = messages_end_ref.cast::<web_sys::HtmlElement>() {
                element.scroll_into_view();
            }
            || ()
        });
    }
    /*
    todo add key here
               props.messages.iter().map(|message| {
                    html! {
                        <MessageBubble key={message.w} message={message.clone()} />
                    }
                }).collect::<Html>()

     */
    html! {
        <div class="flex-1 overflow-y-auto p-4 space-y-4">
            {
                if let Some(messages) = &props.messages {
                    let mut sorted_messages = messages.clone();
                    sorted_messages.sort_by_key(|m| m.timestamp);

                    sorted_messages.iter().map(|message| {
                        html! {
                            <MessageBubble key={message.timestamp.timestamp_micros()} message={message.clone()} />
                        }
                    }).collect::<Html>()
                } else {
                    html! {}
                }
            }
            <div ref={messages_end_ref} />
        </div>
    }
}
