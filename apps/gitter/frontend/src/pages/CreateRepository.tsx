import React, { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { create_repo, set_ssh_key_fingerprint } from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import { validateSshKey } from '../utils/sshKey';

function CreateRepository(): React.JSX.Element {
    const navigate = useNavigate();
    const [repoName, setRepoName] = useState('');
    const [description, setDescription] = useState('');
    const [sshKey, setSshKey] = useState('');
    const [error, setError] = useState('');

    const handleSubmit = async (): Promise<void> => {
        setError('');
        if (sshKey.trim()) {
            const validationError = validateSshKey(sshKey);
            if (validationError) {
                setError(validationError);
                return;
            }
        }
        const txHash = await create_repo(repoName, description);
        await await_tx_inclusion(txHash);
        if (sshKey.trim()) {
            const sshTxHash = await set_ssh_key_fingerprint(repoName, sshKey);
            await await_tx_inclusion(sshTxHash);
        }
        navigate(`/repo/${repoName}`);
    };

    return (
        <div className="max-w-5xl mx-auto px-5 py-5 md:px-6 md:py-6 lg:py-8">
            {/* Breadcrumb */}
            <div className="flex items-center gap-2 mb-4 md:mb-6 text-xs md:text-sm overflow-x-auto scrollbar-thin pb-1">
                <Link to="/" className="text-app-accent-blue hover:underline whitespace-nowrap flex-shrink-0">Repositories</Link>
                <span className="text-app-text-secondary flex-shrink-0">/</span>
                <span className="text-app-text-secondary whitespace-nowrap flex-shrink-0">Create a new repository</span>
            </div>

            {/* Header */}
            <h1 className="text-xl md:text-2xl font-semibold mb-2">Create a new repository</h1>

            <div className="space-y-4 md:space-y-6">
                {/* Repository name */}
                <div>
                    <label htmlFor="repo-name" className="block text-sm font-semibold mb-2">
                        Repository name
                    </label>
                    <input
                        id="repo-name"
                        type="text"
                        value={repoName}
                        onChange={(e) => setRepoName(e.target.value)}
                        className="w-full px-3 py-2 bg-app-bg-secondary border border-app-border rounded-md text-app-text-primary focus:outline-none focus:ring-2 focus:ring-app-accent-blue focus:border-transparent"
                    />
                </div>

                {/* Description */}
                <div>
                    <label htmlFor="description" className="block text-sm font-semibold mb-2">
                        Description <span className="text-app-text-secondary font-normal">(optional)</span>
                    </label>
                    <input
                        id="description"
                        type="text"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        placeholder="A short description of your repository"
                        className="w-full px-3 py-2 bg-app-bg-secondary border border-app-border rounded-md text-app-text-primary focus:outline-none focus:ring-2 focus:ring-app-accent-blue focus:border-transparent"
                    />
                </div>

                {/* SSH Key (optional) */}
                <div>
                    <label htmlFor="ssh-key" className="block text-sm font-semibold mb-2">
                        SSH Public Key <span className="text-app-text-secondary font-normal">(optional)</span>
                    </label>
                    <textarea
                        id="ssh-key"
                        value={sshKey}
                        onChange={(e) => setSshKey(e.target.value)}
                        placeholder="ssh-ed25519 AAAA... user@host"
                        className="w-full px-3 py-2 bg-app-bg-secondary border border-app-border rounded-md text-app-text-primary focus:outline-none focus:ring-2 focus:ring-app-accent-blue focus:border-transparent font-mono text-sm"
                        rows={2}
                    />
                    <p className="text-xs text-app-text-secondary mt-1">
                        Required to push via SSH. You can set this later in repository settings.
                    </p>
                    {error && (
                        <p className="text-sm text-app-accent-red mt-2">{error}</p>
                    )}
                </div>

                {/* Action buttons */}
                <div className="flex items-center gap-3 pt-4 border-t border-app-border">
                    <button
                        type="button"
                        onClick={handleSubmit}
                        className="btn-primary px-4 py-2 disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled={!repoName.trim()}
                    >
                        Create repository
                    </button>
                    <Link
                        to="/"
                        className="btn-secondary px-4 py-2"
                    >
                        Cancel
                    </Link>
                </div>
            </div>
        </div>
    );
}

export default CreateRepository;
