use crate::types::GroupMember;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct GroupMembersSidebarProps {
    pub members: Vec<GroupMember>,
    pub is_open: bool,
    pub on_toggle: Callback<()>,
}

#[function_component(GroupMembersSidebar)]
pub fn group_members_sidebar(props: &GroupMembersSidebarProps) -> Html {
    let toggle = {
        let on_toggle = props.on_toggle.clone();
        Callback::from(move |_: MouseEvent| on_toggle.emit(()))
    };

    html! {
        <div class={classes!(
            "bg-white", "border-l", "border-gray-200", "flex", "flex-col",
            "transition-all", "duration-300", "overflow-hidden",
            "absolute", "sm:relative", "right-0", "z-10", "h-full",
            if props.is_open { "w-full sm:w-80" } else { "w-0" }
        )}>
            // Members Header
            <div class="p-4 border-b border-gray-200 flex items-center justify-between">
                <h2 class="text-lg font-bold text-gray-800">{"Group Members"}</h2>
                <button
                    onclick={toggle.clone()}
                    class="p-2 hover:bg-gray-100 rounded-full transition"
                >
                    <svg class="w-5 h-5 text-gray-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>

            // Members Count
            <div class="px-4 py-3 bg-gray-50 border-b border-gray-200">
                <p class="text-sm text-gray-600">
                    {format!("{} members", props.members.len())}
                </p>
            </div>

            // Members List
            <div class="flex-1 overflow-y-auto">
                {
                    props.members.iter().map(|member| {
                        let status_color = match member.status.as_str() {
                            "online" => "bg-green-500",
                            "away" => "bg-yellow-500",
                            _ => "bg-gray-400",
                        };

                        html! {
                            <div
                                key={member.id}
                                class="flex items-center gap-3 p-4 hover:bg-gray-50 transition"
                            >
                                <div class="relative">
                                    <div class="w-10 h-10 bg-blue-500 rounded-full flex items-center justify-center text-white font-semibold flex-shrink-0">
                                        {&member.avatar}
                                    </div>
                                    // Status indicator
                                    <div class={classes!(
                                        "absolute", "bottom-0", "right-0",
                                        "w-3", "h-3", "rounded-full", "border-2", "border-white",
                                        status_color
                                    )} />
                                </div>
                                <div class="flex-1 min-w-0">
                                    <div class="flex items-center gap-2">
                                        <h3 class="font-semibold text-gray-800 text-sm truncate">{&member.name}</h3>
                                        {
                                            if member.role == "Admin" {
                                                html! {
                                                    <span class="px-2 py-0.5 bg-blue-100 text-blue-700 text-xs rounded-full font-medium">
                                                        {"Admin"}
                                                    </span>
                                                }
                                            } else {
                                                html! {}
                                            }
                                        }
                                    </div>
                                    <p class="text-xs text-gray-500 capitalize">{&member.status}</p>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>

            // Add Member Button
            <div class="p-4 border-t border-gray-200">
                <button class="w-full px-4 py-2 bg-blue-500 hover:bg-blue-600 text-white rounded-lg transition font-medium flex items-center justify-center gap-2">
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                    </svg>
                    {"Add Member"}
                </button>
            </div>
        </div>
    }
}
