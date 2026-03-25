import React, { useState } from 'react';
import Modal from '@/components/common/Modal';
import Avatar from '@/components/common/Avatar';
import { set_display_name } from '../../../wasm/pkg';

interface UserSettingsModalProps {
    isOpen: boolean;
    onClose: () => void;
    pubkey: string;
    currentDisplayName: string;
    onSaved: (newName: string) => void;
}

function UserSettingsModal({ isOpen, onClose, pubkey, currentDisplayName, onSaved }: UserSettingsModalProps): React.JSX.Element | null {
    const [name, setName] = useState(currentDisplayName);
    const [saving, setSaving] = useState(false);

    const handleSave = async () => {
        const trimmed = name.trim();
        if (!trimmed || trimmed === currentDisplayName) return;
        setSaving(true);
        try {
            await set_display_name(trimmed);
            onSaved(trimmed);
        } catch (e) {
            console.error('Failed to set display name:', e);
        } finally {
            setSaving(false);
        }
    };

    return (
        <Modal isOpen={isOpen} onClose={onClose} title="User Settings">
            <div className="px-4 pb-4 flex flex-col items-center gap-4">
                <Avatar identifier={pubkey} name={currentDisplayName || pubkey} size={80} />

                {/* Display name */}
                <div className="w-full">
                    <label className="text-xs font-semibold text-dc-text-muted uppercase tracking-wide">Display Name</label>
                    <input
                        type="text"
                        value={name}
                        onChange={e => setName(e.target.value)}
                        className="mt-1 w-full bg-dc-bg-tertiary text-dc-text rounded px-3 py-2 text-sm outline-none focus:ring-1 focus:ring-dc-blurple"
                        placeholder="Enter display name"
                        maxLength={32}
                    />
                </div>

                <button
                    onClick={handleSave}
                    disabled={saving || !name.trim() || name.trim() === currentDisplayName}
                    className="w-full py-2 bg-dc-blurple hover:bg-dc-blurple-hover text-white text-sm font-medium rounded transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                    {saving ? 'Saving...' : 'Save'}
                </button>
            </div>
        </Modal>
    );
}

export default UserSettingsModal;
