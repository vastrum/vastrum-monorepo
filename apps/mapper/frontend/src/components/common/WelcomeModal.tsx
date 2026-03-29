import React from 'react';
import Modal from './Modal';

interface WelcomeModalProps {
    isOpen: boolean;
    onClose: () => void;
}

function WelcomeModal({ isOpen, onClose }: WelcomeModalProps): React.JSX.Element {
    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Mapper">
            <div className="p-6 space-y-6">
                <div>
                    <p className="text-app-text-secondary">
                        Mapper is an experimental decentralized interactive map hosted on Vastrum.
                    </p>

                    <br />
                    <ul className="list-disc list-inside text-app-text-secondary ml-2 space-y-1">
                        <li>The data comes from the OpenStreetMaps dataset</li>
                        <li>The map tile data is uploaded to Vastrum, the client then reads this data from Vastrum</li>
                        <li>Currently only Monaco is supported</li>
                    </ul>

                    <br />

                </div>

                <div className="flex flex-col gap-1 text-sm">
                    {/* docs */}
                    <a href="https://xpkeuoccopibhnakya3luhrsphalhnqo2ifmxe65murdjft54n3q.vastrum.net/apps/mapper" target="_blank" rel="noopener noreferrer" className="text-app-accent hover:underline">
                        Mapper docs
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
