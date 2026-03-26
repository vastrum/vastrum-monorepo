pub mod chatter;
pub mod components;
pub mod types;

#[function_component(App)]
fn app() -> Html {
    let conversations: UseStateHandle<HashMap<x25519::PublicKey, FrontendConversation>> =
        use_state(HashMap::new);
    let chatter_state = use_mut_ref(|| None::<ChatterState>);
    let is_initialized = use_state(|| false);
    let current_user_name = use_state(|| String::from(""));
    let pending_messages = use_mut_ref(Vec::<(Conversation, String)>::new);

    {
        let chatter_state = chatter_state.clone();
        let is_initialized = is_initialized.clone();
        let current_user_name = current_user_name.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let mut state = ChatterState::init().await;
                let stored_user_name = state.get_current_set_name().await;

                if let Ok(mut borrowed_state) = chatter_state.try_borrow_mut() {
                    *borrowed_state = Some(state);
                    current_user_name.set(stored_user_name);
                    is_initialized.set(true);
                }
            });
            || ()
        });
    }

    let sync_conversations = {
        let conversations = conversations.clone();
        let chatter_state = chatter_state.clone();
        let is_initialized = is_initialized.clone();
        let pending_messages = pending_messages.clone();

        Callback::from(move |_| {
            if !*is_initialized {
                return;
            }

            let conversations = conversations.clone();
            let chatter_state = chatter_state.clone();
            let pending_messages = pending_messages.clone();

            spawn_local(async move {
                if let Ok(mut state) = chatter_state.try_borrow_mut() {
                    if let Some(ref mut s) = *state {
                        let msgs: Vec<_> = pending_messages.borrow_mut().drain(..).collect();
                        for (conversation, message) in msgs {
                            s.send_message_in_conversation(conversation, message).await;
                        }
                        let new_convo = s.get_all_conversations().await;
                        conversations.set(new_convo);
                    }
                }
            });
        })
    };

    {
        let sync_conversations = sync_conversations.clone();
        let is_initialized = is_initialized.clone();

        use_effect_with(*is_initialized, move |is_init| {
            let interval = if *is_init {
                sync_conversations.emit(());
                Some(Interval::new(500, move || {
                    sync_conversations.emit(());
                }))
            } else {
                None
            };

            move || {
                drop(interval);
            }
        });
    }

    let active_conversation_id: UseStateHandle<Option<x25519::PublicKey>> = use_state(|| None);

    let mut frontend_conversation: Option<FrontendConversation> = None;
    if let Some(active_conversation_id) = *active_conversation_id {
        let res = conversations.get(&active_conversation_id);
        if let Some(res) = res {
            frontend_conversation = Some(res.clone());
        }
    }
    let sidebar_open = use_state(|| true);
    let members_sidebar_open = use_state(|| false);
    let show_add_contact_modal = use_state(|| false);
    let show_welcome_modal = use_state(|| true);

    let toggle_sidebar = {
        let sidebar_open = sidebar_open.clone();
        Callback::from(move |_| {
            sidebar_open.set(!*sidebar_open);
        })
    };

    let toggle_members_sidebar = {
        let members_sidebar_open = members_sidebar_open.clone();
        Callback::from(move |_| {
            members_sidebar_open.set(!*members_sidebar_open);
        })
    };

    let on_contact_select = {
        let _conversations = conversations.clone();
        let sidebar_open = sidebar_open.clone();
        let members_sidebar_open = members_sidebar_open.clone();
        let active_conversation_id = active_conversation_id.clone();
        Callback::from(move |conversation_id: x25519::PublicKey| {
            active_conversation_id.set(Some(conversation_id));
            members_sidebar_open.set(false);

            // Close sidebar on mobile when selecting a contact
            if let Some(window) = window() {
                if window.inner_width().ok().and_then(|w| w.as_f64()).unwrap_or(0.0) < 640.0 {
                    sidebar_open.set(false);
                }
            }
        })
    };

    let toggle_add_contact_modal = {
        let show_add_contact_modal = show_add_contact_modal.clone();
        Callback::from(move |_| {
            show_add_contact_modal.set(!*show_add_contact_modal);
        })
    };
    let on_add_contact = {
        let chatter_state = chatter_state.clone();
        let is_initialized = is_initialized.clone();
        let show_add_contact_modal = show_add_contact_modal.clone();

        Callback::from(move |invitation_link: InvitationLink| {
            if !*is_initialized {
                return;
            }

            let chatter_state = chatter_state.clone();
            let show_add_contact_modal = show_add_contact_modal.clone();

            spawn_local(async move {
                if let Ok(mut state) = chatter_state.try_borrow_mut() {
                    if let Some(ref mut s) = *state {
                        s.start_conversation(invitation_link).await;
                        show_add_contact_modal.set(false);
                    }
                }
            });
        })
    };
    let on_send_message = {
        let pending_messages = pending_messages.clone();
        let is_initialized = is_initialized.clone();

        Callback::from(move |(conversation, message): (Conversation, String)| {
            if !*is_initialized {
                return;
            }
            pending_messages.borrow_mut().push((conversation, message));
        })
    };

    let on_set_username = {
        let current_user_name = current_user_name.clone();
        let chatter_state = chatter_state.clone();

        let is_initialized = is_initialized.clone();

        Callback::from(move |name: String| {
            if !*is_initialized {
                return;
            }
            let chatter_state = chatter_state.clone();
            let current_user_name = current_user_name.clone();
            spawn_local(async move {
                current_user_name.set(name.clone());
                if let Ok(mut state) = chatter_state.try_borrow_mut() {
                    if let Some(ref mut s) = *state {
                        s.set_name(name).await;
                    }
                }
            });
        })
    };

    let contacts: Vec<FrontendConversation> = conversations.values().cloned().collect();

    html! {
        <>
            <style> {include_str!("../generated/tailwind.css")} </style>
            <div class="flex h-screen bg-gray-100">
                <Sidebar
                    contacts={contacts}
                    is_open={*sidebar_open}
                    on_toggle={toggle_sidebar.clone()}
                    on_contact_select={on_contact_select}
                    on_add_contact_modal_open={toggle_add_contact_modal.clone()}
                    active_conversation_id={*active_conversation_id}
                />
                <AddContactModal
                    is_open={*show_add_contact_modal}
                    on_close={toggle_add_contact_modal}
                    on_add={on_add_contact}
                    on_set_username={on_set_username}
                    current_user_name={current_user_name.to_string()}

                />
                <ConversationBox
                    conversation={frontend_conversation}
                    on_toggle_sidebar={toggle_sidebar}
                    on_toggle_members={toggle_members_sidebar}
                    on_send_message={on_send_message}
                />

            </div>
            <WelcomeModal
                is_open={*show_welcome_modal}
                on_close={{
                    let show_welcome_modal = show_welcome_modal.clone();
                    Callback::from(move |_| show_welcome_modal.set(false))
                }}
            />
        </>
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<App>::new().render();
}

use crate::{
    chatter::chatter_state::{ChatterState, Conversation, FrontendConversation, InvitationLink},
    components::conversation_box::ConversationBox,
};
use components::{add_contact_modal::AddContactModal, sidebar::Sidebar, welcome_modal::WelcomeModal};
use gloo_timers::callback::Interval;
use vastrum_shared_types::crypto::x25519;
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::window;
use yew::{platform::spawn_local, prelude::*};
