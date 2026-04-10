import React from 'react';
import Modal from './Modal';

interface WelcomeModalProps {
    isOpen: boolean;
    onClose: () => void;
}

function WelcomeModal({ isOpen, onClose }: WelcomeModalProps): React.JSX.Element {
    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Gitter">
            <div className="p-6 space-y-6">
                <div>
                    <p className="text-app-text-secondary">
                        Gitter is an experimental decentralized Git forge hosted on Vastrum.
                    </p>

                    <br />
                    <p className="text-app-text-secondary">
                        Currently you can
                    </p>
                    <ul className="list-disc list-inside text-app-text-secondary ml-2 space-y-1">
                        <li>Clone via HTTPS and push via SSH using any standard git client</li>
                        <li>Fork and submit pull requests using the Gitter frontend</li>
                        <li>Create issues and discussions</li>
                        <li>Merge pull requests locally inside the frontend (No CLI required)</li>
                    </ul>

                    <br />
                    <p className="text-app-text-secondary">
                        For more details, check Gitter docs.
                    </p>

                </div>

                <div className="flex flex-col gap-1 text-sm">
                    <a href="https://xpkeuoccopibhnakya3luhrsphalhnqo2ifmxe65murdjft54n3q.vastrum.net/apps/gitter" target="_blank" rel="noopener noreferrer" className="text-app-accent hover:underline">
                        Gitter docs
                    </a>
                    <a href="https://xpkeuoccopibhnakya3luhrsphalhnqo2ifmxe65murdjft54n3q.vastrum.net/" target="_blank" rel="noopener noreferrer" className="text-app-accent hover:underline">
                        Vastrum docs
                    </a>
                </div>

                {/* Close Button */}
                <div className="flex justify-end pt-2">
                    <button onClick={onClose} className="btn-primary">
                        Close
                    </button>
                </div>
            </div>
        </Modal>
    );
}

export default WelcomeModal;
