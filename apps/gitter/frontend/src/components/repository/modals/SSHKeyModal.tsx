import React, { useState } from 'react';
import { Key } from 'lucide-react';
import Modal from '../../common/Modal';
import { set_ssh_key_fingerprint } from '../../../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import { validateSshKey } from '../../../utils/sshKey';

interface SSHKeyModalProps {
    isOpen: boolean;
    onClose: () => void;
    repositoryName: string;
    onRefresh: () => void;
}

function SSHKeyModal({ isOpen, onClose, repositoryName, onRefresh }: SSHKeyModalProps): React.JSX.Element {
    const [sshKey, setSshKey] = useState('');
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState('');

    const handleSave = async (): Promise<void> => {
        setError('');
        const validationError = validateSshKey(sshKey);
        if (validationError) {
            setError(validationError);
            return;
        }
        setLoading(true);
        try {
            const txHash = await set_ssh_key_fingerprint(repositoryName, sshKey);
            await await_tx_inclusion(txHash);
            onRefresh();
            onClose();
            setSshKey('');
        } finally {
            setLoading(false);
        }
    };

    const handleClose = () => {
        setSshKey('');
        setError('');
        onClose();
    };

    return (
        <Modal isOpen={isOpen} onClose={handleClose} title="Repository Settings">
            <div className="p-6">
                <div className="mb-4">
                    <label htmlFor="ssh-key" className="block text-sm font-medium text-app-text-primary mb-2">
                        SSH Public Key
                    </label>
                    <p className="text-sm text-app-text-secondary mb-3">
                        Paste your SSH public key to enable push access via the git relay.
                        Supports any key type (ed25519, RSA, ECDSA).
                    </p>
                    <textarea
                        id="ssh-key"
                        value={sshKey}
                        onChange={(e) => setSshKey(e.target.value)}
                        className="w-full px-3 py-2 bg-app-bg-secondary border border-app-border rounded-md text-app-text-primary focus:outline-none focus:ring-2 focus:ring-app-accent-blue focus:border-transparent font-mono text-sm"
                        placeholder="ssh-ed25519 AAAA... user@host"
                        rows={3}
                        autoFocus
                    />
                    <p className="text-xs text-app-text-secondary mt-1">
                        Don't have an SSH key?{' '}
                        <a
                            href="https://docs.github.com/en/authentication/connecting-to-github-with-ssh/generating-a-new-ssh-key-and-adding-it-to-the-ssh-agent"
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-app-accent-blue hover:underline"
                        >
                            Generate one
                        </a>
                    </p>
                    {error && (
                        <p className="text-sm text-app-accent-red mt-2">{error}</p>
                    )}
                </div>

                <div className="flex justify-end gap-3">
                    <button
                        type="button"
                        onClick={handleClose}
                        className="btn-secondary"
                    >
                        Cancel
                    </button>
                    <button
                        type="button"
                        onClick={handleSave}
                        className="btn-primary flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled={!sshKey.trim() || loading}
                    >
                        <Key className="w-4 h-4" />
                        {loading ? 'Saving...' : 'Save SSH Key'}
                    </button>
                </div>
            </div>
        </Modal>
    );
}

export default SSHKeyModal;
