use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ChatHeaderProps {
    pub contact: Option<String>,
    pub member_count: usize,
    pub on_toggle_sidebar: Callback<()>,
    pub on_toggle_members: Callback<()>,
}

#[function_component(ChatHeader)]
pub fn chat_header(props: &ChatHeaderProps) -> Html {
    let toggle_sidebar = {
        let on_toggle = props.on_toggle_sidebar.clone();
        Callback::from(move |_: MouseEvent| on_toggle.emit(()))
    };

    let toggle_members = {
        let on_toggle = props.on_toggle_members.clone();
        Callback::from(move |_: MouseEvent| on_toggle.emit(()))
    };
    let is_group = false;
    html! {
        <div class="bg-white border-b border-gray-200 px-4 py-3 flex items-center justify-between shadow-sm">
            <div class="flex items-center gap-3">
                <button
                    onclick={toggle_sidebar}
                    class="sm:hidden p-2 hover:bg-gray-100 rounded-full"
                >
                    <svg class="w-6 h-6 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                    </svg>
                </button>


                {
                    if let Some(contact) = &props.contact {
                        let mut contact = contact.clone();
                        //max limit on name
                        contact.truncate(15);
                        html! {
                            <>
                                <div class={classes!(
                                    "w-10", "h-10", "rounded-full", "flex", "items-center",
                                    "justify-center", "text-white", "font-semibold",
                                    if is_group { "bg-green-500" } else { "bg-blue-500" }
                                )}>
                                    {contact.chars().next().unwrap_or('A').to_ascii_uppercase()}
                                </div>
                                <div>
                                    <h2 class="font-semibold text-gray-800">{contact}</h2>
                                    <p class="text-xs text-gray-500">
                                        {if is_group {
                                            format!("{} members", props.member_count)
                                        } else {
                                            "".to_string()
                                        }}
                                    </p>
                                </div>
                            </>
                        }
                    } else {
                        html! {}
                    }
                }





            </div>


            <div class="flex items-center gap-2">
                {

                    if is_group {
                        html! {
                            <button
                                onclick={toggle_members}
                                class="hover:bg-gray-100 p-2 rounded-full transition"
                                title="View Members"
                            >
                                <svg class="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0zm6 3a2 2 0 11-4 0 2 2 0 014 0zM7 10a2 2 0 11-4 0 2 2 0 014 0z" />
                                </svg>
                            </button>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>




        </div>
    }
}
