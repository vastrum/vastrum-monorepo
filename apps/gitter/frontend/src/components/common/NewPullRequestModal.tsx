import React, { useState, useEffect } from 'react';
import { GitPullRequest, ChevronDown } from 'lucide-react';
import Modal from './Modal';
import MarkdownEditor from './MarkdownEditor';
import { get_forks_by_me_of_this_repo, create_pull_request, get_repo_default_branch } from '../../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';

interface NewPullRequestModalProps {
    isOpen: boolean;
    onClose: () => void;
    baseRepository: string;
    baseOwner: string;
    onRefresh: () => void;
}

function NewPullRequestModal({ isOpen, onClose, baseRepository, baseOwner, onRefresh }: NewPullRequestModalProps): React.JSX.Element {
    const [title, setTitle] = useState('');
    const [description, setDescription] = useState('');
    const [selectedFork, setSelectedFork] = useState('');
    const [forkedRepos, setForkedRepos] = useState<string[]>([]);

    useEffect(() => {
        const fetchForks = async (): Promise<void> => {
            if (!isOpen) return;
            const forks = await get_forks_by_me_of_this_repo(baseRepository);
            setForkedRepos(forks);
        };
        fetchForks();
    }, [isOpen, baseRepository]);

    const handleSubmit = async (): Promise<void> => {
        const [baseBranch, headBranch] = await Promise.all([
            get_repo_default_branch(baseRepository),
            get_repo_default_branch(selectedFork),
        ]);
        const txHash = await create_pull_request(
            baseRepository,
            baseBranch,
            selectedFork,
            headBranch,
            title,
            description,
        );
        onClose();
        setTitle('');
        setDescription('');
        setSelectedFork('');
        await await_tx_inclusion(txHash);
        onRefresh();
    };

    const handleClose = () => {
        setTitle('');
        setDescription('');
        setSelectedFork('');
        onClose();
    };

    return (
        <Modal isOpen={isOpen} onClose={handleClose} title="Create a new pull request">
            <div className="p-6">
                <div className="space-y-5">
                    {/* Base Repository Info */}
                    <div className="bg-app-bg-tertiary border border-app-border rounded-lg p-4">
                        <div className="flex items-center gap-3 text-sm">
                            <GitPullRequest className="w-5 h-5 text-app-text-secondary" />
                            <div>
                                <p className="text-app-text-secondary text-xs">Base repository:</p>
                                <p className="font-semibold text-app-text-primary">
                                    {baseRepository}
                                </p>
                            </div>
                        </div>
                    </div>

                    {/* Fork Selection */}
                    <div>
                        <label htmlFor="fork-select" className="block text-sm font-medium text-app-text-primary mb-2">
                            From your fork <span className="text-app-accent-red">*</span>
                        </label>
                        <div className="relative">
                            <select
                                id="fork-select"
                                value={selectedFork}
                                onChange={(e) => setSelectedFork(e.target.value)}
                                className="input-field appearance-none pr-10"
                            >
                                <option value="">Select a forked repository...</option>
                                {forkedRepos.map((fork) => (
                                    <option key={fork} value={fork}>
                                        {fork}
                                    </option>
                                ))}
                            </select>
                            <ChevronDown className="absolute right-3 top-1/2 -translate-y-1/2 w-4 h-4 text-app-text-secondary pointer-events-none" />
                        </div>
                        <p className="text-xs text-app-text-secondary mt-1">
                            Select which of your forks you want to create a pull request from
                        </p>
                    </div>

                    {/* Pull Request Title */}
                    <div>
                        <label htmlFor="pr-title" className="block text-sm font-medium text-app-text-primary mb-2">
                            Title <span className="text-app-accent-red">*</span>
                        </label>
                        <input
                            type="text"
                            id="pr-title"
                            value={title}
                            onChange={(e) => setTitle(e.target.value)}
                            className="input-field"
                            placeholder="Brief description of your changes"
                        />
                    </div>

                    {/* Pull Request Description */}
                    <div>
                        <label className="block text-sm font-medium text-app-text-primary mb-2">
                            Description
                        </label>
                        <MarkdownEditor
                            value={description}
                            onChange={setDescription}
                            placeholder="Describe the changes you've made and why..."
                            minHeight="120px"
                        />
                    </div>
                </div>

                {/* Action Buttons */}
                <div className="flex justify-end gap-3 mt-6">
                    <button
                        type="button"
                        onClick={handleClose}
                        className="btn-secondary"
                    >
                        Cancel
                    </button>
                    <button
                        type="button"
                        onClick={handleSubmit}
                        className="btn-primary flex items-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed"
                        disabled={!title.trim() || !selectedFork}
                    >
                        <GitPullRequest className="w-4 h-4" />
                        Create pull request
                    </button>
                </div>
            </div>
        </Modal>
    );
}

export default NewPullRequestModal;
