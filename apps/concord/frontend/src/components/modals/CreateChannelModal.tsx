import React, { useState } from 'react';
import Modal from '@/components/common/Modal';
import { create_channel } from '../../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';

interface CreateChannelModalProps {
    isOpen: boolean;
    onClose: () => void;
    serverId: number;
    onCreated: () => void;
}

function CreateChannelModal({ isOpen, onClose, serverId, onCreated }: CreateChannelModalProps): React.JSX.Element {
    const [name, setName] = useState('');
    const [loading, setLoading] = useState(false);

    const handleSubmit = async () => {
        if (!name.trim() || loading) return;
        setLoading(true);
        const channelName = name.trim().toLowerCase().replace(/\s+/g, '-').replace(/[^a-z0-9-]/g, '');
        const txHash = await create_channel(BigInt(serverId), channelName);
        setName('');
        await await_tx_inclusion(txHash);
        setLoading(false);
        onCreated();
    };

    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Create Channel">
            <div className="p-4 space-y-4">
                <div>
                    <label className="text-xs font-semibold uppercase text-dc-text-muted block mb-2">Channel Name</label>
                    <div className="flex items-center bg-dc-bg-tertiary rounded px-3">
                        <span className="text-dc-text-muted mr-1">#</span>
                        <input
                            type="text"
                            value={name}
                            onChange={e => setName(e.target.value)}
                            placeholder="new-channel"
                            className="w-full bg-transparent text-dc-text py-2 text-sm outline-none"
                        />
                    </div>
                </div>
                <div className="flex justify-end gap-3 pt-2">
                    <button onClick={onClose} className="px-4 py-2 text-sm text-dc-text-muted hover:text-dc-text transition-colors">
                        Cancel
                    </button>
                    <button
                        onClick={handleSubmit}
                        disabled={!name.trim() || loading}
                        className="px-4 py-2 text-sm bg-dc-blurple hover:bg-dc-blurple-hover text-white rounded transition-colors disabled:opacity-50"
                    >
                        {loading ? 'Creating...' : 'Create Channel'}
                    </button>
                </div>
            </div>
        </Modal>
    );
}

export default CreateChannelModal;
