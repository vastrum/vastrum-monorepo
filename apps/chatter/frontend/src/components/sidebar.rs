use crate::{chatter::chatter_state::FrontendConversation, components::contact_list::ContactList};
use vastrum_shared_types::crypto::x25519;
use web_sys::HtmlInputElement;
use yew::prelude::*;

//todo add #[derive(Properties, PartialEq)] for performance reasons when re rendering

#[derive(Properties, PartialEq)]
pub struct SidebarProps {
    pub contacts: Vec<FrontendConversation>,
    pub is_open: bool,
    pub on_toggle: Callback<()>,
    pub on_contact_select: Callback<x25519::PublicKey>,
    pub on_add_contact_modal_open: Callback<()>,
    pub active_conversation_id: Option<x25519::PublicKey>,
}

#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let search_query = use_state(String::new);

    let toggle = {
        let on_toggle = props.on_toggle.clone();
        Callback::from(move |_: MouseEvent| on_toggle.emit(()))
    };

    let add_contact = {
        let on_add_contact = props.on_add_contact_modal_open.clone();
        Callback::from(move |_: MouseEvent| on_add_contact.emit(()))
    };

    let on_search_input = {
        let search_query = search_query.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            search_query.set(input.value().to_lowercase());
        })
    };

    /*
       // Filter contacts based on search query
       let filtered_contacts: Vec<Contact> = if search_query.is_empty() {
           props.contacts.clone()
       } else {
           props
               .contacts
               .iter()
               .filter(|contact| contact.name.to_lowercase().contains(&*search_query))
               .cloned()
               .collect()
       };
    */
    html! {
        <div class={classes!(
            "bg-white", "border-r", "border-gray-200", "flex", "flex-col",
            "transition-all", "duration-300", "overflow-hidden", "sm:relative",
            "absolute", "z-20", "h-full",
            if props.is_open { "w-full sm:w-80" } else { "w-0" }
        )}>
            // Sidebar Header
            <div class="p-4 border-b border-gray-200">
                <div class="flex items-center justify-between mb-4">
                    <h1 class="text-xl font-bold text-gray-800">{"Chatter"}</h1>
                    <div class="flex items-center gap-2">
                        <button
                            onclick={add_contact}
                            class="p-2 hover:bg-gray-100 rounded-full transition"
                            title="Add Contact"
                        >
                            <svg class="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                            </svg>
                        </button>
                        <button
                            onclick={toggle.clone()}
                            class="sm:hidden p-2 hover:bg-gray-100 rounded-full"
                        >
                            <svg class="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                </div>
                <div class="relative">
                    <svg class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                    </svg>
                    <input
                        type="text"
                        placeholder="Search"
                        value={(*search_query).clone()}
                        oninput={on_search_input}
                        class="w-full pl-10 pr-10 py-2 bg-gray-100 rounded-full text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                    {
                        if !search_query.is_empty() {
                            let clear_search = {
                                let search_query = search_query.clone();
                                Callback::from(move |_: MouseEvent| {
                                    search_query.set(String::new());
                                })
                            };
                            html! {
                                <button
                                    onclick={clear_search}
                                    class="absolute right-3 top-1/2 transform -translate-y-1/2 p-1 hover:bg-gray-200 rounded-full transition"
                                >
                                    <svg class="w-4 h-4 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                                    </svg>
                                </button>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            </div>

            <ContactList
                contacts={props.contacts.clone()}
                on_select={props.on_contact_select.clone()}
                on_add_contact_modal_open={props.on_add_contact_modal_open.clone()}
                active_conversation_id={props.active_conversation_id}
            />
        </div>
    }
}
