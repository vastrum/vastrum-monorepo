use crate::chatter::chatter_state::FrontendConversation;
use chrono::{DateTime, Duration, Local, Utc};
use vastrum_shared_types::crypto::x25519;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ContactListProps {
    pub contacts: Vec<FrontendConversation>,
    pub on_select: Callback<x25519::PublicKey>,
    pub on_add_contact_modal_open: Callback<()>,
    pub active_conversation_id: Option<x25519::PublicKey>,
}
fn relative_time(utc_time: &DateTime<Utc>) -> String {
    let local_time = utc_time.with_timezone(&Local);
    let now = Local::now();
    let diff = now.signed_duration_since(local_time);

    if diff < Duration::minutes(1) {
        "Just now".to_string()
    } else if diff < Duration::hours(1) {
        format!("{} minutes ago", diff.num_minutes())
    } else if diff < Duration::days(1) {
        format!("{} hours ago", diff.num_hours())
    } else {
        local_time.format("%b %d, %I:%M %p").to_string()
    }
}
#[function_component(ContactList)]
pub fn contact_list(props: &ContactListProps) -> Html {
    html! {
        <div class="flex-1 overflow-y-auto">
            {
                if props.contacts.is_empty() {
                    html! {
                        <div class="flex flex-col items-center justify-center py-12 px-4 text-center">
                            <svg class="w-16 h-16 text-gray-300 mb-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                            </svg>
                            <p class="text-sm text-gray-500">{"No contacts found"}</p>
                        </div>
                    }
                } else {
                    let mut sorted_contacts = props.contacts.clone();
                    //sort by latest message
                    //and secondary contact key
                    sorted_contacts.sort_by(|a, b| {
                        let a_time = a.messages.iter().map(|m| m.timestamp).max().unwrap_or(DateTime::<Utc>::MIN_UTC);
                        let b_time = b.messages.iter().map(|m| m.timestamp).max().unwrap_or(DateTime::<Utc>::MIN_UTC);

                        b_time.cmp(&a_time)
                            .then_with(|| a.conversation.contact_pub_key.to_string()
                            .cmp(&b.conversation.contact_pub_key.to_string()))
                    });

                    sorted_contacts.iter().map(|contact| {
                        let mut last_message = "".to_string();
                        let mut last_message_time = "".to_string();
                        let latest_message = contact.messages.iter().max_by_key(|m| m.timestamp);
                        if let Some(latest_message) = latest_message {
                            last_message = latest_message.content.clone();
                            last_message_time = relative_time(&latest_message.timestamp);
                        }


                        let contact_id_string = contact.conversation.contact_pub_key.to_string();
                        let contact_id = contact.conversation.contact_pub_key;
                        let on_select = props.on_select.clone();
                        let mut active_contact = false;
                        let contact_is_group = false;


                        if let Some(active_conversation_id) = props.active_conversation_id {
                            if contact_id == active_conversation_id {
                                active_contact = true;
                            }
                        }
                        let mut name = contact.conversation.contact_name.clone();

                        //max limit on name
                        name.truncate(15);
                        let time = last_message_time;
                        let last_message = last_message;
                        html! {
                            <div
                                key={contact_id_string}
                                onclick={move |_| on_select.emit(contact_id)}
                                class={classes!(
                                    "flex", "items-center", "gap-3", "p-4",
                                    "hover:bg-gray-50", "cursor-pointer", "transition",
                                    if active_contact { "bg-blue-50" } else { "" }
                                )}
                            >
                                <div class="relative">
                                    <div class={classes!(
                                        "w-12", "h-12", "rounded-full", "flex", "items-center",
                                        "justify-center", "text-white", "font-semibold", "flex-shrink-0",
                                        if contact_is_group { "bg-green-500" } else { "bg-blue-500" }
                                    )}>
                                        {name.chars().next().unwrap_or('A').to_ascii_uppercase()}
                                    </div>
                                </div>
                                <div class="flex-1 min-w-0">
                                    <div class="flex items-center justify-between mb-1">
                                        <h3 class="font-semibold text-gray-800 truncate">{name}</h3>
                                        <span class="text-xs text-gray-500 flex-shrink-0 ml-2">{time}</span>
                                    </div>
                                    <p class="text-sm text-gray-500 truncate">{last_message}</p>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            }
        </div>
    }
}
