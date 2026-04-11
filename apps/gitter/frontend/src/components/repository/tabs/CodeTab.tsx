import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import {
    Code,
    FileText,
    Folder,
    GitBranch,
} from 'lucide-react';
import { type GetRepoDetail, type ExplorerEntry } from '../../../../wasm/pkg';
import CloneModal from '../../common/CloneModal';
import MarkdownRenderer from '../../common/MarkdownRenderer';
import EmptyRepoGuide from '../EmptyRepoGuide';
import { generateAvatarGradient } from '../../../utils/avatarGenerator';

interface CodeTabProps {
    repoData: GetRepoDetail;
    onBranchChange: (branch: string) => void;
}

const CodeTab = ({ repoData, onBranchChange }: CodeTabProps): React.JSX.Element => {
    const [showCloneModal, setShowCloneModal] = useState(false);
    const { git_repo, head_commit_author_name, head_commit_message, head_commit_hash, readme_contents, branches, current_branch } = repoData;

    const topLevelEntries: ExplorerEntry[] = repoData.top_level_files;
    const isEmpty = head_commit_hash === "";

    return (
        <div className="lg:col-span-2">
            {isEmpty ? (
                /* Empty repo: show getting-started guide */
                <div className="bg-app-bg-secondary border border-app-border rounded-lg overflow-hidden mb-6">
                    <div className="flex items-center justify-between p-4 border-b border-app-border">
                        <h2 className="text-lg font-semibold text-app-text-primary">Quick setup</h2>
                        <button
                            onClick={() => setShowCloneModal(true)}
                            className="btn-primary text-sm flex items-center gap-2 whitespace-nowrap"
                        >
                            <Code className="w-4 h-4" />
                            Clone
                        </button>
                    </div>
                    <EmptyRepoGuide repositoryName={git_repo.name} />
                </div>
            ) : (
                /* Repo with commits: show file list + README */
                <>
                    <div className="bg-app-bg-secondary border border-app-border rounded-lg overflow-hidden mb-6">
                        {/* Branch selector and actions */}
                        <div className="flex items-center justify-between p-4 border-b border-app-border gap-6 md:gap-8">
                            <div className="flex items-center gap-2 md:gap-3 flex-1 min-w-0">
                                {/* Branch selector */}
                                <div className="flex items-center gap-2 flex-shrink-0">
                                    <GitBranch className="w-4 h-4 text-app-text-secondary" />
                                    <select
                                        value={current_branch}
                                        onChange={(e) => onBranchChange(e.target.value)}
                                        className="bg-app-bg-tertiary border border-app-border rounded-md px-2 py-1 text-sm text-app-text-primary focus:outline-none focus:ring-2 focus:ring-app-accent-blue"
                                    >
                                        {branches.map((b) => (
                                            <option key={b} value={b}>{b}</option>
                                        ))}
                                    </select>
                                </div>
                                {/* Latest commit info */}
                                <div className="flex items-baseline gap-3 min-w-0 flex-1">
                                    <div className={`w-5 h-5 rounded-full bg-gradient-to-br ${generateAvatarGradient(head_commit_author_name)} flex-shrink-0 self-center`} />
                                    <span className="text-sm font-semibold text-app-text-primary flex-shrink-0">
                                        {head_commit_author_name}
                                    </span>
                                    <span className="text-sm text-app-text-secondary truncate min-w-0">
                                        {head_commit_message}
                                    </span>
                                    <span className="hidden sm:inline font-mono text-sm text-app-text-secondary flex-shrink-0">{git_repo.head_commit_hash.slice(0, 7)}</span>
                                </div>
                            </div>

                            <div className="flex items-center gap-2 flex-shrink-0">
                                <button
                                    onClick={() => setShowCloneModal(true)}
                                    className="btn-primary text-sm flex items-center gap-2 whitespace-nowrap"
                                >
                                    <Code className="w-4 h-4" />
                                    Clone
                                </button>
                            </div>
                        </div>

                        {/* File list - flat, no expansion */}
                        <div>
                            {topLevelEntries.map((entry, index) => (
                                <Link
                                    key={`${entry.oid}-${index}`}
                                    to={`/repo/${git_repo.name}/tree/${entry.name}`}
                                    className="file-item"
                                >
                                    {entry.is_directory ? (
                                        <Folder className="w-4 h-4 text-app-accent-blue flex-shrink-0" />
                                    ) : (
                                        <FileText className="w-4 h-4 text-app-text-secondary flex-shrink-0" />
                                    )}
                                    <span className="flex-1 font-medium text-app-text-primary">{entry.name}</span>
                                </Link>
                            ))}
                        </div>
                    </div>

                    {/* README */}
                    <div className="bg-app-bg-secondary border border-app-border rounded-lg overflow-hidden">
                        <div className="flex items-center gap-2 px-4 py-3 border-b border-app-border">
                            <FileText className="w-4 h-4" />
                            <span className="font-semibold">README.md</span>
                        </div>
                        <div className="p-6">
                            {readme_contents ? (
                                <MarkdownRenderer content={readme_contents} />
                            ) : (
                                <div className="text-app-text-secondary">No README found.</div>
                            )}
                        </div>
                    </div>
                </>
            )}

            {/* Clone Modal */}
            <CloneModal
                isOpen={showCloneModal}
                onClose={() => setShowCloneModal(false)}
                repositoryName={git_repo.name}
            />
        </div>
    );
};

export default CodeTab;
