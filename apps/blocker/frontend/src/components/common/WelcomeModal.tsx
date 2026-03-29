import React from 'react';
import Modal from './Modal';

interface WelcomeModalProps {
    isOpen: boolean;
    onClose: () => void;
}

function WelcomeModal({ isOpen, onClose }: WelcomeModalProps): React.JSX.Element {
    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Blocker">
            <div className="p-6 space-y-6">
                <div>
                    <p className="text-app-text-secondary">
                        Blocker is an experimental blockchain explorer for the Vastrum blockchain.
                    </p>

                    <br />
                    <p className="text-app-text-secondary">
                        It is fully hosted onchain on Vastrum, using the regular siteKV database that all other apps uses.
                    </p>

                    <br />
                    <p className="text-app-text-secondary">
                        Currently it supports
                    </p>
                    <ul className="list-disc list-inside text-app-text-secondary ml-2 space-y-1">
                        <li>Current block height</li>
                        <li>Latest blocks</li>
                        <li>Block details</li>
                        <li>Latest transactions</li>
                        <li>Sites deployed</li>
                        <li>Sites details</li>
                        <li>Account details</li>
                        <li>Transaction detail</li>
                    </ul>
                </div>

                <div className="flex flex-col gap-1 text-sm">
                    {/* docs */}
                    <a href="https://xpkeuoccopibhnakya3luhrsphalhnqo2ifmxe65murdjft54n3q.vastrum.net/apps/blocker" target="_blank" rel="noopener noreferrer" className="text-app-accent hover:underline">
                        Blocker docs
                    </a>
                    {/* docs */}
                    <a href="https://xpkeuoccopibhnakya3luhrsphalhnqo2ifmxe65murdjft54n3q.vastrum.net/" target="_blank" rel="noopener noreferrer" className="text-app-accent hover:underline">
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
