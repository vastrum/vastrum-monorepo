import React from 'react';
import { Terminal } from 'lucide-react';
import Modal from './Modal';

interface CloneModalProps {
    isOpen: boolean;
    onClose: () => void;
    repositoryName: string;
    isOwner: boolean;
}

interface UrlBlockProps {
    icon: React.ReactNode;
    title: string;
    command: string;
    note: string;
}

function UrlBlock({ icon, title, command, note }: UrlBlockProps): React.JSX.Element {
    return (
        <div className="border border-app-border rounded-lg p-4 bg-app-bg-tertiary">
            <h3 className="font-semibold text-app-text-primary mb-2 flex items-center gap-2">
                {icon}
                {title}
            </h3>
            <div className="bg-app-bg-primary border border-app-border rounded px-3 py-2 font-mono text-xs text-app-accent-green break-all">
                {command}
            </div>
            <p className="mt-2 text-xs text-app-text-secondary">{note}</p>
        </div>
    );
}

function CloneModal({ isOpen, onClose, repositoryName, isOwner }: CloneModalProps): React.JSX.Element {
    const sshBlock = (
        <UrlBlock
            icon={<Terminal className="w-4 h-4" />}
            title="Clone & push via SSH"
            command={`git clone ssh://git@gitrelay.vastrum.org:2222/${repositoryName}`}
            note={
                isOwner
                    ? 'Handles both clone and push. Requires your SSH key to be registered in repo settings.'
                    : 'For repository maintainers. Pushing requires a registered SSH key.'
            }
        />
    );

    const httpsBlock = (
        <UrlBlock
            icon={<Terminal className="w-4 h-4" />}
            title="HTTPS clone (read-only)"
            command={`git clone https://gitrelay.vastrum.org/${repositoryName}`}
            note="Read-only. Pushing over HTTPS will fail, use SSH to push."
        />
    );

    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Clone this repository">
            <div className="p-6 space-y-4">
                <p className="text-sm text-app-text-secondary">
                    Clone this repository using any standard git client.
                </p>

                {isOwner ? (
                    <>
                        {sshBlock}
                        {httpsBlock}
                    </>
                ) : (
                    <>
                        {httpsBlock}
                        {sshBlock}
                    </>
                )}

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
