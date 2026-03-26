import React from 'react';
import { Link } from 'react-router-dom';
import { Code } from 'lucide-react';

type TabType = 'code' | 'issues' | 'pulls' | 'discussions';

interface RepositoryTabsProps {
    repoId: string;
    activeTab: TabType;
    issueCount: number;
    prCount: number;
    discussionCount: number;
}

function RepositoryTabs({ repoId, activeTab, issueCount, prCount, discussionCount }: RepositoryTabsProps): React.JSX.Element {
    return (
        <div className="border-b border-app-border mb-4 md:mb-6">
            <div className="flex gap-0 overflow-x-auto scrollbar-thin">
                <Link
                    to={`/repo/${repoId}/code`}
                    className={`tab-item whitespace-nowrap ${activeTab === 'code' ? 'active' : ''}`}
                >
                    <Code className="w-4 h-4 inline mr-2" />
                    Code
                </Link>
                <Link
                    to={`/repo/${repoId}/issues`}
                    className={`tab-item whitespace-nowrap ${activeTab === 'issues' ? 'active' : ''}`}
                >
                    Issues
                    <span className="ml-1 md:ml-2 bg-app-bg-tertiary px-1.5 md:px-2 py-0.5 rounded-full text-xs font-semibold">{issueCount}</span>
                </Link>
                <Link
                    to={`/repo/${repoId}/pulls`}
                    className={`tab-item whitespace-nowrap ${activeTab === 'pulls' ? 'active' : ''}`}
                >
                    <span className="hidden sm:inline">Pull requests</span>
                    <span className="sm:hidden">PRs</span>
                    <span className="ml-1 md:ml-2 bg-app-bg-tertiary px-1.5 md:px-2 py-0.5 rounded-full text-xs font-semibold">{prCount}</span>
                </Link>
                <Link
                    to={`/repo/${repoId}/discussions`}
                    className={`tab-item whitespace-nowrap ${activeTab === 'discussions' ? 'active' : ''}`}
                >
                    <span className="hidden sm:inline">Discussions</span>
                    <span className="sm:hidden">Discuss</span>
                    <span className="ml-1 md:ml-2 bg-app-bg-tertiary px-1.5 md:px-2 py-0.5 rounded-full text-xs font-semibold">{discussionCount}</span>
                </Link>
            </div>
        </div>
    );
}

export default RepositoryTabs;
