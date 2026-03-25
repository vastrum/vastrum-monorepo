import React, { useState } from 'react';
import Modal from '../../common/Modal';
import MarkdownEditor from '../../common/MarkdownEditor';
import { create_issue } from '../../../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';

interface NewIssueModalProps {
    isOpen: boolean;
    onClose: () => void;
    repoId: string;
    onRefresh: () => void;
}

function NewIssueModal({ isOpen, onClose, repoId, onRefresh }: NewIssueModalProps): React.JSX.Element {
    const [title, setTitle] = useState('');
    const [description, setDescription] = useState('');

    const handleSubmit = async (): Promise<void> => {
        const txHash = await create_issue(title, description, repoId);
        onClose();
        setTitle('');
        setDescription('');
        await await_tx_inclusion(txHash);
        onRefresh();
    };

    return (
        <Modal isOpen={isOpen} onClose={onClose} title="New Issue">
            {/* Modal Body */}
            <div className="p-6 space-y-4">
                {/* Title Input */}
                <div>
                    <label htmlFor="issue-title" className="block text-sm font-medium mb-2">
                        Title
                    </label>
                    <input
                        id="issue-title"
                        type="text"
                        value={title}
                        onChange={(e) => setTitle(e.target.value)}
                        placeholder="Enter issue title"
                        className="w-full bg-app-bg-primary border border-app-border rounded-md px-4 py-2 text-app-text-primary focus:outline-none focus:ring-2 focus:ring-app-accent-blue"
                    />
                </div>

                {/* Description */}
                <div>
                    <label className="block text-sm font-medium mb-2">
                        Description
                    </label>
                    <MarkdownEditor
                        value={description}
                        onChange={setDescription}
                        placeholder="Describe the issue..."
                        minHeight="200px"
                    />
                </div>
            </div>

            {/* Modal Footer */}
            <div className="flex items-center justify-end gap-3 p-6 border-t border-app-border">
                <button
                    type="button"
                    onClick={onClose}
                    className="btn-secondary"
                >
                    Cancel
                </button>
                <button
                    type="button"
                    onClick={handleSubmit}
                    disabled={!title.trim()}
                    className="btn-primary disabled:opacity-50 disabled:cursor-not-allowed"
                >
                    Create issue
                </button>
            </div>
        </Modal>
    );
}

export default NewIssueModal;
