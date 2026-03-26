use gloo_timers::callback::Timeout;
use wasm_bindgen::prelude::*;
use web_sys::HtmlInputElement;
use yew::{platform::spawn_local, prelude::*};

use crate::chatter::chatter_state::{
    InvitationLink, get_conversation_invitation_link, parse_conversation_invitation_link,
};

#[wasm_bindgen(inline_js = "export function copy_to_clipboard(text) { navigator.clipboard.writeText(text); }")]
extern "C" {
    fn copy_to_clipboard(text: &str);
}

#[derive(Properties, PartialEq)]
pub struct AddContactModalProps {
    pub is_open: bool,
    pub on_close: Callback<()>,
    pub on_add: Callback<InvitationLink>,
    pub on_set_username: Callback<String>,
    pub current_user_name: String,
}

#[function_component(AddContactModal)]
pub fn add_contact_modal(props: &AddContactModalProps) -> Html {
    let entered_invitation_link = use_state(|| None);
    let invite_link_textbox_value = use_state(String::new);
    let set_name_textbox_value = use_state(|| props.current_user_name.clone());

    {
        let set_name_textbox_value = set_name_textbox_value.clone();
        let current_user_name = props.current_user_name.clone();
        use_effect_with(current_user_name.clone(), move |user_name| {
            set_name_textbox_value.set(user_name.clone());
            || ()
        });
    }

    let copied = use_state(|| false);
    let own_invite_link = use_state(|| None::<String>);
    {
        let invite_link = own_invite_link.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                let link = get_conversation_invitation_link().await;
                invite_link.set(Some(link));
            });
            || ()
        });
    }

    let on_close = {
        let on_close = props.on_close.clone();
        Callback::from(move |_: MouseEvent| on_close.emit(()))
    };

    let on_copy_invite = {
        let own_invite_link = own_invite_link.clone();
        let copied = copied.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(link) = &*own_invite_link {
                copy_to_clipboard(link);
                copied.set(true);
                let copied = copied.clone();
                Timeout::new(2000, move || copied.set(false)).forget();
            }
        })
    };

    let on_invitation_link_input = {
        let entered_invitation_link = entered_invitation_link.clone();
        let invite_link_textbox_value = invite_link_textbox_value.clone();
        let own_invite_link = own_invite_link.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            let input_value = input.value();
            invite_link_textbox_value.set(input.value());

            if let Some(own_invite_link) = &*own_invite_link {
                let is_own_invite_link = input_value == *own_invite_link;
                if is_own_invite_link {
                    entered_invitation_link.set(None);
                    return;
                }
            }

            let parsed = parse_conversation_invitation_link(input_value);

            if let Some(parsed) = parsed {
                entered_invitation_link.set(Some(parsed));
            } else {
                entered_invitation_link.set(None);
            }
        })
    };

    let add_contact = {
        let entered_invitation_link = entered_invitation_link.clone();
        let invite_link_textbox_value = invite_link_textbox_value.clone();
        let on_add = props.on_add.clone();
        Callback::from(move |_: MouseEvent| {
            if let Some(link) = *entered_invitation_link {
                on_add.emit(link);
                entered_invitation_link.set(None);
                invite_link_textbox_value.set(String::new());
            }
        })
    };

    let on_set_name_input = {
        let set_name_textbox_value = set_name_textbox_value.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            set_name_textbox_value.set(input.value());
        })
    };
    let update_username = {
        let set_name_textbox_value = set_name_textbox_value.clone();
        let on_set_username = props.on_set_username.clone();
        Callback::from(move |_: MouseEvent| {
            on_set_username.emit(set_name_textbox_value.to_string());
            set_name_textbox_value.set(String::new());
        })
    };
    if !props.is_open {
        return html! {};
    }

    html! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
            <div class="bg-white rounded-lg shadow-xl p-6 w-full max-w-md mx-4">
                <div class="flex items-center justify-between mb-4">
                    <h2 class="text-xl font-bold text-gray-800">{"Add Contact and settings"}</h2>
                    <button
                        onclick={on_close.clone()}
                        class="p-1 hover:bg-gray-100 rounded-full transition"
                    >
                        <svg class="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>



                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                        {"Set username"}
                    </label>
                    <input
                        type="text"
                        oninput={on_set_name_input}
                        value={(*set_name_textbox_value).clone()}
                        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                </div>

                <div class="flex gap-3 justify-end">
                    <button
                        onclick={update_username}
                        class={classes!(
                            "px-4", "py-2", "rounded-lg", "transition", "bg-blue-500", "hover:bg-blue-600", "text-white"
                        )}
                    >
                        {"Update username"}
                    </button>
                </div>


                <br/>

                <h2>{"Your invite link (Share this to start a chat)"}</h2>
                <p class="font-bold text-gray-800 break-all">
                    { if let Some(invite_link) = &*own_invite_link {
                        html! { <p>{ invite_link }</p> }
                    } else {
                        html! {}
                    }}
                </p>
                { if own_invite_link.is_some() {
                    html! {
                        <div class="flex gap-3 justify-end">
                            <button
                                onclick={on_copy_invite}
                                class={classes!(
                                    "px-4", "py-2", "rounded-lg", "transition", "text-white",
                                    if *copied { "bg-green-500" } else { "bg-blue-500 hover:bg-blue-600" }
                                )}
                            >
                                { if *copied { "Copied!" } else { "Copy Invite Link" } }
                            </button>
                        </div>
                    }
                } else {
                    html! {}
                }}

                <br/>


                <div class="mb-4">
                    <label class="block text-sm font-medium text-gray-700 mb-2">
                        {"Enter invite link"}
                    </label>
                    <input
                        type="text"
                        oninput={on_invitation_link_input}
                        value={(*invite_link_textbox_value).clone()}
                        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                </div>


                <div class="flex gap-3 justify-end">
                    <button
                        onclick={add_contact}
                        disabled={entered_invitation_link.is_none()}
                        class={classes!(
                            "px-4", "py-2", "rounded-lg", "transition",
                            if entered_invitation_link.is_some() {
                                "bg-blue-500 hover:bg-blue-600 text-white"
                            } else {
                                "bg-gray-300 text-gray-500 cursor-not-allowed"
                            }
                        )}
                    >
                        {"Add Contact"}
                    </button>
                </div>

                <br/>





            </div>
        </div>
    }
}
