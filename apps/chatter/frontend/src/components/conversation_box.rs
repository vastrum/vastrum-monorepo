use crate::{
    chatter::chatter_state::{Conversation, FrontendConversation},
    components::{chat_header::ChatHeader, message_input::MessageInput, message_list::MessageList},
};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ConversationBoxProps {
    pub conversation: Option<FrontendConversation>,
    pub on_toggle_sidebar: Callback<()>,
    pub on_toggle_members: Callback<()>,
    pub on_send_message: Callback<(Conversation, String)>,
}

#[function_component(ConversationBox)]
pub fn conversation_box(props: &ConversationBoxProps) -> Html {
    let is_group_chat = false;

    let mut contact_name = None;
    if let Some(convo) = &props.conversation {
        contact_name = Some(convo.conversation.contact_name.clone());
    }
    html! {
        <>
            <div class="flex-1 flex flex-col">

                <ChatHeader
                    contact={contact_name}
                    member_count={0}
                    on_toggle_sidebar={props.on_toggle_sidebar.clone()}
                    on_toggle_members={props.on_toggle_members.clone()}
                />


                {
                    if let Some(conversation) = &props.conversation {
                        html!{
                            <>
                                <MessageList messages={conversation.messages.clone()} />
                                <MessageInput on_send_message={props.on_send_message.clone()} conversation={conversation.conversation.clone()} />
                            </>
                        }
                    } else {
                        html! {}
                    }
                }

            </div>

            {
                if is_group_chat {
                   html!{}
                   /* html! {
                        <GroupMembersSidebar
                            members={current_group_members}
                            is_open={*members_sidebar_open}
                            on_toggle={&toggle_members_sidebar}
                        />
                    } */
                } else {
                    html! {}
                }
            }
            </>
    }
}
