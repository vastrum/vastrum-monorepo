import React from 'react';
import { GitFork } from 'lucide-react';
import { type GitRepository } from '../../../wasm/pkg';

interface RepositoryHeaderProps {
    repository: GitRepository;
    onFork: () => void;
}

function RepositoryHeader({ repository, onFork }: RepositoryHeaderProps): React.JSX.Element {
    return (
        <div className="mb-6">
            <div className="flex items-center gap-2 mb-4">
                <h1 className="text-xl font-semibold">
                    <span>{repository.name}</span>
                </h1>
            </div>

            <p className="text-app-text-secondary mb-4">{repository.description}</p>

            <div className="flex items-center justify-between flex-wrap gap-4">
                <div className="flex items-center gap-4">
                    <span className="badge badge-open">
                        Public
                    </span>
                </div>

                {/* Action Buttons */}
                <div className="flex items-center gap-2">
                    <button
                        onClick={onFork}
                        className="flex items-center gap-2 px-4 py-1.5 bg-app-bg-tertiary border border-app-border rounded-md hover:bg-app-hover transition-colors text-sm font-medium"
                    >
                        <GitFork className="w-4 h-4" />
                        <span>Fork</span>
                    </button>
                </div>
            </div>
        </div>
    );
}

export default RepositoryHeader;
