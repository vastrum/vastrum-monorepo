import React, { useState } from 'react';
import Modal from '@/components/common/Modal';
import { create_server } from '../../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';

interface CreateServerModalProps {
    isOpen: boolean;
    onClose: () => void;
    onCreated: () => void;
}

function CreateServerModal({ isOpen, onClose, onCreated }: CreateServerModalProps): React.JSX.Element {
    const [name, setName] = useState('');
    const [loading, setLoading] = useState(false);

    const handleSubmit = async () => {
        if (!name.trim() || loading) return;
        setLoading(true);
        const txHash = await create_server(name.trim());
        setName('');
        await await_tx_inclusion(txHash);
        setLoading(false);
        onCreated();
    };

    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Create a Server">
            <div className="p-4 space-y-4">
                <div>
                    <label className="text-xs font-semibold uppercase text-dc-text-muted block mb-2">Server Name</label>
                    <input
                        type="text"
                        value={name}
                        onChange={e => setName(e.target.value)}
                        placeholder="Server name"
                        className="w-full bg-dc-bg-tertiary text-dc-text rounded px-3 py-2 text-sm outline-none focus:ring-2 focus:ring-dc-blurple"
                    />
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
                        {loading ? 'Creating...' : 'Create'}
                    </button>
                </div>
            </div>
        </Modal>
    );
}

export default CreateServerModal;
