import React from 'react';
import Modal from './Modal';

interface WelcomeModalProps {
    isOpen: boolean;
    onClose: () => void;
}

function WelcomeModal({ isOpen, onClose }: WelcomeModalProps): React.JSX.Element {
    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Letterer">
            <div className="p-6 space-y-6">
                <div>
                    <p className="text-app-text-secondary">
                        Letterer is a prototype for a decentralized online document editor alternative.
                    </p>

                    <br />
                    <p className="text-app-text-secondary">
                        It currently supports
                    </p>
                    <ul className="list-disc list-inside text-app-text-secondary ml-2 space-y-1">
                        <li>Creating documents</li>
                        <li>Basic functionality for editing text documents</li>
                        <li>Saving the documents to vastrum with encryption</li>
                        <li>Sharing your documents with others</li>
                        <li>No real time editing supported</li>
                    </ul>

                    <br />
                    <p className="text-app-text-secondary">
                        Currently only a very limited document editor is supported (no sheets, no slides).
                    </p>

                    <br />
                    <p className="text-app-text-secondary">
                        In order to persist your document, the document is encrypted and then uploaded to keyvalue storage on Vastrum.
                    </p>

                    <br />
                    <p className="text-app-text-secondary">
                        You can share your created documents by creating a share link, which is the private key to the document.
                    </p>
                    <br />
                </div>

                <div className="flex flex-col gap-1 text-sm">
                    <a href="https://xpkeuoccopibhnakya3luhrsphalhnqo2ifmxe65murdjft54n3q.vastrum.net/apps/letterer" target="_blank" rel="noopener noreferrer" className="text-app-accent hover:underline">
                        Letterer docs
                    </a>
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
