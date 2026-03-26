import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { GitFork } from 'lucide-react';
import Modal from './Modal';
import { fork_repo, get_default_fork_name } from '../../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';

interface ForkModalProps {
    isOpen: boolean;
    onClose: () => void;
    repositoryName: string;
}

function ForkModal({ isOpen, onClose, repositoryName }: ForkModalProps): React.JSX.Element {
    const navigate = useNavigate();
    const [newRepoName, setNewRepoName] = useState("");
    const [loading, setLoading] = useState(false);

    useEffect(() => {
        if (isOpen) {
            setLoading(true);
            get_default_fork_name(repositoryName).then((name: string) => {
                setNewRepoName(name);
                setLoading(false);
            });
        }
    }, [isOpen, repositoryName]);

    const handleFork = async (): Promise<void> => {
        const txHash = await fork_repo(newRepoName, repositoryName);
        onClose();
        await await_tx_inclusion(txHash);
        navigate(`/repo/${newRepoName}`);
    };

    const handleClose = () => {
        setNewRepoName("");
        onClose();
    };

    return (
        <Modal isOpen={isOpen} onClose={handleClose} title="Fork this repository">
            <div className="p-6">
                <div className="mb-4">
                    <p className="text-sm text-app-text-secondary mb-4">
                        By creating a fork you create personal copy of this repository, you can independently develop within the fork and then submit the changes to the main branch as a pull request.
                    </p>

                    <div className="bg-app-bg-tertiary border border-app-border rounded-lg p-4 mb-4">
                        <div className="flex items-center gap-3 text-sm">
                            <GitFork className="w-5 h-5 text-app-text-secondary" />
                            <div>
                                <p className="text-app-text-secondary">Forking from:</p>
                                <p className="font-semibold text-app-text-primary">
                                    {repositoryName}
                                </p>
                            </div>
                        </div>
                    </div>

                    <label htmlFor="repo-name" className="block text-sm font-medium text-app-text-primary mb-2">
                        Repository name
                    </label>
                    <input
                        type="text"
                        id="repo-name"
                        value={newRepoName}
                        onChange={(e) => setNewRepoName(e.target.value)}
                        className="input-field"
                        placeholder={loading ? "Loading..." : "Enter repository name"}
                        autoFocus
                        disabled={loading}
                    />
                    <p className="text-xs text-app-text-secondary mt-1">
                        Your fork will be created as: <span className="font-mono">{newRepoName}</span>
                    </p>
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
                        onClick={handleFork}
                        className="btn-primary flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled={!newRepoName.trim() || loading}
                    >
                        <GitFork className="w-4 h-4" />
                        Create fork
                    </button>
                </div>
            </div>
        </Modal>
    );
}

export default ForkModal;
