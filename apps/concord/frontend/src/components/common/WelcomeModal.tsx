import React from 'react';

interface WelcomeModalProps {
    isOpen: boolean;
    onClose: () => void;
}

function WelcomeModal({ isOpen, onClose }: WelcomeModalProps): React.JSX.Element | null {
    if (!isOpen) {
        return null;
    }

    return (
        <div className="fixed inset-0 flex items-center justify-center z-50 p-4">
            <div className="bg-app-bg-secondary border border-app-border rounded-lg max-w-full md:max-w-2xl lg:max-w-4xl w-full max-h-[90vh] overflow-hidden relative">
                {/* Modal Header */}
                <div className="flex items-center justify-between px-4 py-3 md:p-6 border-b border-app-border bg-app-bg-secondary">
                    <h2 className="text-lg md:text-xl font-semibold text-app-text-primary">Concord</h2>
                    <button
                        type="button"
                        onClick={onClose}
                        className="text-app-text-secondary hover:text-app-text-primary transition-colors"
                    >
                        <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                {/* Modal Content */}
                <div className="overflow-y-auto scrollbar-thin max-h-[calc(90vh-88px)]">
                    <div className="p-6 space-y-6">
                        <div>
                            <p className="text-app-text-secondary">
                                Concord is a prototype for a decentralized chat application.
                            </p>

                            <br />
                            <p className="text-app-text-secondary">
                                Currently it supports
                            </p>
                            <ul className="list-disc list-inside text-app-text-secondary ml-2 space-y-1">
                                <li>Creating servers</li>
                                <li>Creating channels in servers</li>
                                <li>Sharing invite links to your server</li>
                                <li>Sending messages in channels</li>
                                <li>DMs</li>
                                <li>Basic moderation</li>
                            </ul>

                            <br />
                            <p className="text-app-text-secondary">
                                The content of all messages are encrypted. For more details see Concord docs.
                            </p>
                            <br />
                        </div>

                        <div className="flex flex-col gap-1 text-sm">
                            <a href="https://docs.vastrum.net/apps/concord" target="_blank" rel="noopener noreferrer" className="text-app-accent hover:underline">
                                Concord docs
                            </a>
                            <a href="https://docs.vastrum.net/" target="_blank" rel="noopener noreferrer" className="text-app-accent hover:underline">
                                Vastrum docs
                            </a>
                        </div>

                        {/* Close Button */}
                        <div className="flex justify-end pt-2">
                            <button onClick={onClose} className="bg-app-accent text-white px-4 py-2 rounded-md font-medium hover:opacity-80 transition-colors">
                                Close
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default WelcomeModal;
