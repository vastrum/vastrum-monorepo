import React from 'react';
import { type GitRepository } from '../../../wasm/pkg';

interface RepositorySidebarProps {
    repository: GitRepository;
}

function RepositorySidebar({ repository }: RepositorySidebarProps): React.JSX.Element {
    return (
        <aside className="hidden lg:block space-y-4 md:space-y-6">
            <div className="bg-app-bg-secondary border border-app-border rounded-lg p-3 md:p-4">
                <h3 className="font-semibold mb-3 md:mb-4">About</h3>
                <p className="text-app-text-secondary text-sm mb-3 md:mb-4">
                    {repository.description}
                </p>
            </div>
        </aside>
    );
}

export default RepositorySidebar;
