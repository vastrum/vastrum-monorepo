import React, { useState, useEffect } from 'react';
import { useLocation, useParams } from 'react-router-dom';
import { get_repo_page_data, GetRepoDetail } from '../../wasm/pkg';
import PullRequestsTab from '../components/repository/tabs/PullRequestsTab';
import DiscussionsTab from '../components/repository/tabs/DiscussionsTab';
import IssuesTab from '../components/repository/tabs/IssuesTab';
import CodeTab from '../components/repository/tabs/CodeTab';
import RepositoryHeader from '../components/repository/RepositoryHeader';
import RepositoryTabs from '../components/repository/RepositoryTabs';
import RepositorySidebar from '../components/repository/RepositorySidebar';
import ForkModal from '../components/common/ForkModal';
import SSHKeyModal from '../components/repository/modals/SSHKeyModal';

type TabType = 'code' | 'issues' | 'pulls' | 'discussions';

function Repository(): React.JSX.Element {
    const { repoId } = useParams<{ repoId: string }>();
    const location = useLocation();
    const [showForkModal, setShowForkModal] = useState(false);
    const [showSettingsModal, setShowSettingsModal] = useState(false);
    const [repoData, setRepoData] = useState<GetRepoDetail | null>(null);
    const [loading, setLoading] = useState(true);
    const [selectedBranch, setSelectedBranch] = useState<string | undefined>(undefined);

    const fetchRepoData = async (branch?: string): Promise<void> => {
        if (!repoId) return;
        try {
            const data = await get_repo_page_data(repoId, branch);
            setRepoData(data);
            setSelectedBranch(data.current_branch);
        } catch (error) {
            console.error('Failed to fetch repository data:', error);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchRepoData();
    }, [repoId]);

    const handleBranchChange = (branch: string): void => {
        fetchRepoData(branch);
    };

    if (loading) {
        return (
            <div className="max-w-7xl mx-auto px-5 py-5 md:px-6 md:py-6">
                <div className="text-app-text-secondary">Loading repository...</div>
            </div>
        );
    }

    // If repository not found, redirect to all repositories
    if (!repoData) {
        return (<p>Repo not found</p>);
    }

    // Determine active tab from URL path
    const getActiveTab = (): TabType => {
        const path = location.pathname;
        if (path.includes('/issues')) return 'issues';
        if (path.includes('/pulls')) return 'pulls';
        if (path.includes('/discussions')) return 'discussions';
        return 'code'; // Default for '/repo/:id' and '/repo/:id/code'
    };

    const activeTab = getActiveTab();

    const { git_repo, issue_count, pr_count, discussion_count, is_owner } = repoData;

    return (
        <div className="max-w-7xl mx-auto px-5 py-5 md:px-6 md:py-6">
            <RepositoryHeader
                repository={git_repo}
                isOwner={is_owner}
                onFork={() => setShowForkModal(true)}
                onSettings={() => setShowSettingsModal(true)}
            />

            <RepositoryTabs
                repoId={git_repo.name}
                activeTab={activeTab}
                issueCount={Number(issue_count)}
                prCount={Number(pr_count)}
                discussionCount={Number(discussion_count)}
            />

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-5 md:gap-6">
                {activeTab === 'code' && <CodeTab repoData={repoData} onBranchChange={handleBranchChange} />}

                {/* Issues Tab */}
                {activeTab === 'issues' && <IssuesTab repoId={git_repo.name} />}

                {/* Pull Requests Tab */}
                {activeTab === 'pulls' && <PullRequestsTab repoId={git_repo.name} repoOwner={git_repo.owner} />}

                {/* Discussions Tab */}
                {activeTab === 'discussions' && <DiscussionsTab repoId={git_repo.name} />}

                <RepositorySidebar repository={git_repo} />
            </div>

            <ForkModal
                isOpen={showForkModal}
                onClose={() => setShowForkModal(false)}
                repositoryName={git_repo.name}
            />

            <SSHKeyModal
                isOpen={showSettingsModal}
                onClose={() => setShowSettingsModal(false)}
                repositoryName={git_repo.name}
                onRefresh={fetchRepoData}
            />
        </div>
    );
}

export default Repository;
