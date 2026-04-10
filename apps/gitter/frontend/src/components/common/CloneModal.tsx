import React from 'react';
import { Terminal } from 'lucide-react';
import Modal from './Modal';

interface CloneModalProps {
    isOpen: boolean;
    onClose: () => void;
    repositoryName: string;
}

function CloneModal({ isOpen, onClose, repositoryName }: CloneModalProps): React.JSX.Element {
    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Clone this repository">
            <div className="p-6 space-y-6">
                <div>
                    <p className="text-sm text-app-text-secondary">
                        Clone this repository using any standard git client.
                    </p>
                </div>

                <div className="border border-app-border rounded-lg p-4 bg-app-bg-tertiary">
                    <div className="flex items-start gap-3">
                        <div className="flex-1">
                            <h3 className="font-semibold text-app-text-primary mb-2 flex items-center gap-2">
                                <Terminal className="w-4 h-4" />
                                Clone via HTTPS
                            </h3>
                            <div className="bg-app-bg-primary border border-app-border rounded px-3 py-2 font-mono text-xs text-app-accent-green break-all">
                                git clone https://gitrelay.vastrum.org/{repositoryName}
                            </div>
                        </div>
                    </div>
                </div>

                <div className="flex justify-end">
                    <button onClick={onClose} className="btn-primary">
                        Close
                    </button>
                </div>
            </div>
        </Modal>
    );
}

export default CloneModal;
