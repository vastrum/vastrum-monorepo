import React, { useEffect, useState } from 'react';
import Modal from '@/components/common/Modal';
import { get_server_key_hex } from '../../../wasm/pkg';

interface InviteModalProps {
    isOpen: boolean;
    onClose: () => void;
    serverId: number;
    serverName: string;
}

function InviteModal({ isOpen, onClose, serverId, serverName }: InviteModalProps): React.JSX.Element {
    const [serverKeyHex, setServerKeyHex] = useState<string | null>(null);
    const [copied, setCopied] = useState(false);

    useEffect(() => {
        if (isOpen) {
            get_server_key_hex(BigInt(serverId)).then(key => setServerKeyHex(key ?? null));
        }
    }, [isOpen, serverId]);

    // concord
    const inviteLink = serverKeyHex
        ? `https://x647757zpbejyzxcw7ruqcju32otdmi7vphrg36vhhzglkjccaqq.vastrum.net/join/${serverId}/${serverKeyHex}`
        : `https://x647757zpbejyzxcw7ruqcju32otdmi7vphrg36vhhzglkjccaqq.vastrum.net/join/${serverId}`;

    const handleCopy = () => {
        navigator.clipboard.writeText(inviteLink);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <Modal isOpen={isOpen} onClose={onClose} title={`Invite to ${serverName}`}>
            <div className="p-4 space-y-4">
                <p className="text-sm text-dc-text-muted">
                    Share this link with others so they can join this server.
                </p>
                <div className="bg-dc-bg-tertiary rounded px-3 py-2 text-dc-text text-sm font-mono break-all select-all">
                    {inviteLink}
                </div>
                <button
                    onClick={handleCopy}
                    className={`w-full py-2 rounded text-sm font-medium transition-colors ${
                        copied
                            ? 'bg-green-600 text-white'
                            : 'bg-dc-blurple text-white hover:bg-dc-blurple-hover'
                    }`}
                >
                    {copied ? 'Copied!' : 'Copy'}
                </button>
            </div>
        </Modal>
    );
}

export default InviteModal;
