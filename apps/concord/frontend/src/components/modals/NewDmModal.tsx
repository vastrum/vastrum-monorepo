import React, { useState } from 'react';
import Modal from '@/components/common/Modal';
import { send_dm } from '../../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';

interface NewDmModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSent: () => void;
}

function NewDmModal({ isOpen, onClose, onSent }: NewDmModalProps): React.JSX.Element {
    const [recipient, setRecipient] = useState('');
    const [message, setMessage] = useState('');
    const [loading, setLoading] = useState(false);

    const handleSend = async () => {
        if (!recipient.trim() || !message.trim() || loading) return;
        setLoading(true);
        const txHash = await send_dm(recipient.trim(), message.trim());
        setRecipient('');
        setMessage('');
        await await_tx_inclusion(txHash);
        setLoading(false);
        onSent();
    };

    return (
        <Modal isOpen={isOpen} onClose={onClose} title="New Direct Message">
            <div className="p-4 space-y-4">
                <div>
                    <label className="text-xs font-semibold uppercase text-dc-text-muted block mb-2">Recipient Public Key</label>
                    <input
                        type="text"
                        value={recipient}
                        onChange={e => setRecipient(e.target.value)}
                        placeholder="Paste public key"
                        className="w-full bg-dc-bg-tertiary text-dc-text rounded px-3 py-2 text-sm outline-none font-mono focus:ring-2 focus:ring-dc-blurple"
                    />
                </div>
                <div>
                    <label className="text-xs font-semibold uppercase text-dc-text-muted block mb-2">Message</label>
                    <input
                        type="text"
                        value={message}
                        onChange={e => setMessage(e.target.value)}
                        placeholder=""
                        className="w-full bg-dc-bg-tertiary text-dc-text rounded px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-dc-blurple"
                    />
                </div>
                <div className="flex justify-end gap-3 pt-2">
                    <button onClick={onClose} className="px-4 py-2 text-sm text-dc-text-muted hover:text-dc-text transition-colors">
                        Cancel
                    </button>
                    <button
                        onClick={handleSend}
                        disabled={!recipient.trim() || !message.trim() || loading}
                        className="px-4 py-2 text-sm bg-dc-blurple hover:bg-dc-blurple-hover text-white rounded transition-colors disabled:opacity-50"
                    >
                        {loading ? 'Sending...' : 'Send'}
                    </button>
                </div>
            </div>
        </Modal>
    );
}

export default NewDmModal;
