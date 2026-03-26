import React from 'react';
import Modal from './Modal';

interface WelcomeModalProps {
    isOpen: boolean;
    onClose: () => void;
}

function WelcomeModal({ isOpen, onClose }: WelcomeModalProps): React.JSX.Element {
    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Concourse">
            <div className="p-6 space-y-6">
                <div>
                    <p className="text-app-text-secondary">
                        Concourse is an experimental decentralized forum hosted on Vastrum.
                    </p>

                    <br />
                    <p className="text-app-text-secondary">
                        It currently supports
                    </p>
                    <ul className="list-disc list-inside text-app-text-secondary ml-2 space-y-1">
                        <li>Creating posts</li>
                        <li>Replying to posts</li>
                        <li>Basic markdown</li>
                        <li>Moderation, deletions</li>
                    </ul>
                </div>

                <div className="flex flex-col gap-1 text-sm">
                    <a href="https://docs.vastrum.net/apps/concourse" target="_blank" rel="noopener noreferrer" className="text-app-accent hover:underline">
                        Concourse docs
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
        </Modal>
    );
}

export default WelcomeModal;
