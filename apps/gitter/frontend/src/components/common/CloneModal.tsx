import React from 'react';
import { Terminal, Download } from 'lucide-react';
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
                {/* Introduction */}
                <div>
                    <p className="text-sm text-app-text-secondary">
                        This is a git repository hosted on vastrum, a decentralized website hosting system.
                    </p>
                    <br />

                    <p className="text-sm text-app-text-secondary">
                        To clone this repository you need to download a CLI tool that extends your local git installation,
                        this will allow you to read and write git repositories hosted on the vastrum network.
                    </p>
                </div>

                {/* Step 1: Install Git */}
                <div className="border border-app-border rounded-lg p-4 bg-app-bg-tertiary">
                    <div className="flex items-start gap-3 mb-3">
                        <div className="w-6 h-6 rounded-full bg-app-accent-blue text-white flex items-center justify-center text-sm font-semibold flex-shrink-0">
                            1
                        </div>
                        <div className="flex-1">
                            <h3 className="font-semibold text--app-text-primary mb-2 flex items-center gap-2">
                                <Download className="w-4 h-4" />
                                Install vastrum-cli
                            </h3>

                            <div className="space-y-2 text-sm">
                                <div className="flex items-center gap-2">
                                    <code className="bg-app-bg-primary px-2 py-1 rounded font-mono text-xs">curl -sSf https://raw.githubusercontent.com/vastrum/vastrum-monorepo/HEAD/tooling/cli/install.sh | sh</code>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Step 2: Clone with HTTPS */}
                <div className="border border-app-border rounded-lg p-4 bg-app-bg-tertiary">
                    <div className="flex items-start gap-3">
                        <div className="w-6 h-6 rounded-full bg-app-accent-blue text-white flex items-center justify-center text-sm font-semibold flex-shrink-0">
                            2
                        </div>
                        <div className="flex-1">
                            <h3 className="font-semibold text-app-text-primary mb-2 flex items-center gap-2">
                                <Terminal className="w-4 h-4" />
                                Clone the repository
                            </h3>
                            <div className="mb-4">
                                <div className="bg-app-bg-primary border border-app-border rounded px-3 py-2 font-mono text-xs text-app-accent-green break-all">
                                    vastrum-cli vastrum-git-clone {repositoryName}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>


                {/* Close Button */}
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
