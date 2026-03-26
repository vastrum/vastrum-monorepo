use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct WelcomeModalProps {
    pub is_open: bool,
    pub on_close: Callback<()>,
}

#[function_component(WelcomeModal)]
pub fn welcome_modal(props: &WelcomeModalProps) -> Html {
    let on_close = {
        let on_close = props.on_close.clone();
        Callback::from(move |_: MouseEvent| on_close.emit(()))
    };

    if !props.is_open {
        return html! {};
    }

    html! {
        <div class="fixed inset-0 flex items-center justify-center z-50 p-4">
            <div class="bg-app-bg-secondary border-2 border-app-border rounded-lg max-w-full md:max-w-2xl lg:max-w-4xl w-full max-h-[90vh] overflow-hidden relative shadow-[0_20px_60px_rgba(0,0,0,0.12)]">
                // Modal Header
                <div class="flex items-center justify-between px-4 py-3 border-b border-app-border bg-app-bg-secondary">
                    <h2 class="text-lg font-semibold text-app-text-primary">{"Chatter"}</h2>
                    <button
                        onclick={on_close.clone()}
                        class="text-app-text-secondary hover:text-app-text-primary transition-colors"
                    >
                        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                // Modal Content
                <div class="overflow-y-auto scrollbar-thin max-h-[calc(90vh-88px)]">
                    <div class="p-6 space-y-6">
                        <div>
                            <p class="text-app-text-secondary">
                                {"Chatter is an experimental prototype for a no-metadata private messaging application."}
                            </p>

                            <br />
                            <p class="text-app-text-secondary">{"Currently you can"}</p>
                            <ul class="list-disc list-inside text-app-text-secondary ml-2 space-y-1">
                                <li>{"Start a conversation by sharing your invite link"}</li>
                                <li>{"Group chats are currently not supported, however they are planned"}</li>
                            </ul>

                            <br />
                            <p class="text-app-text-secondary">
                                {"E2E encrypted messaging applications encrypt the messages and hide the contents, however often the applications leak metadata as the server needs to route messages between users."}
                            </p>

                            <br />
                            <p class="text-app-text-secondary">
                                {"This means metadata about which identities communicate with each other is leaked."}
                            </p>

                            <br />
                            <p class="text-app-text-secondary">
                                {"Chatter solves this by creating a private inbox."}
                            </p>

                            <br />
                            <p class="text-app-text-secondary">
                                {"An user can send a message to it without revealing any metadata about the sender or the receiver of the message."}
                            </p>

                            <br />
                        </div>

                        <div class="flex flex-col gap-1 text-sm">
                            <a href="https://docs.vastrum.net/apps/chatter" target="_blank" rel="noopener noreferrer" class="text-app-accent hover:underline">
                                {"Chatter docs"}
                            </a>
                            <a href="https://docs.vastrum.net/" target="_blank" rel="noopener noreferrer" class="text-app-accent hover:underline">
                                {"Vastrum docs"}
                            </a>
                        </div>

                        // Close Button
                        <div class="flex justify-end pt-2">
                            <button
                                onclick={on_close}
                                class="bg-app-accent text-white px-4 py-2 rounded-md font-medium hover:opacity-80 transition-colors"
                            >
                                {"Close"}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
