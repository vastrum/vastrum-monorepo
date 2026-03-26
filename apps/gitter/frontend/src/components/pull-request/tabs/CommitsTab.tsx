import React from 'react';
import { type FrontendCommit } from '../../../../wasm/pkg';
import { generateAvatarGradient } from '../../../utils/avatarGenerator';
import { formatRelativeTime } from '../../../utils/timeUtils';

interface CommitsTabProps {
    commits: FrontendCommit[];
    sourceBranch?: string;
}

function CommitsTab({ commits = [], sourceBranch }: CommitsTabProps): React.JSX.Element {
    return (
        <div className="lg:col-span-2">
            {/* Commits Header */}
            <div className="bg-app-bg-secondary border border-app-border rounded-lg overflow-hidden mb-4">
                <div className="px-4 py-3 border-b border-app-border">
                    <h3 className="font-semibold">
                        Commits {sourceBranch && <span className="text-app-text-secondary font-normal">from {sourceBranch}</span>}
                    </h3>
                </div>

                {/* Commits List */}
                <div>
                    {commits.map((commit, index) => (
                        <div
                            key={index}
                            className="block p-4 border-b border-app-border last:border-b-0"
                        >
                            <div className="flex items-start gap-3">
                                <div className={`w-10 h-10 rounded-full bg-gradient-to-br ${generateAvatarGradient(commit.author_name)} flex-shrink-0`} />
                                <div className="flex-1 min-w-0">
                                    <div className="font-medium text-app-text-primary mb-1">
                                        {commit.message}
                                    </div>
                                    <div className="flex items-center gap-3 text-sm text-app-text-secondary">
                                        <span className="font-semibold text-app-text-primary">
                                            {commit.author_name}
                                        </span>
                                        <span>committed {formatRelativeTime(commit.author_timestamp)}</span>
                                    </div>
                                </div>
                            </div>

                            {/* Commit details */}
                            {/*commit.parent_ids.length > 0 && (
                                <div className="mt-3 ml-13 flex items-center gap-4 text-xs text-app-text-secondary">
                                    <span className="flex items-center gap-1">
                                        <svg viewBox="0 0 16 16" className="w-3 h-3 fill-current">
                                            <path d="M1.5 3.25a2.25 2.25 0 1 1 3 2.122v5.256a2.251 2.251 0 1 1-1.5 0V5.372A2.25 2.25 0 0 1 1.5 3.25Z"></path>
                                        </svg>
                                        {commit.parent_ids.length} {commit.parent_ids.length === 1 ? 'parent' : 'parents'}
                                    </span>
                                </div>
                            )*/}

                        </div>
                    ))}
                </div>
            </div>

            {commits.length === 0 && (
                <div className="bg-app-bg-secondary border border-app-border rounded-lg p-8 text-center">
                    <p className="text-app-text-secondary">No commits found</p>
                </div>
            )}
        </div>
    );
}

export default CommitsTab;
