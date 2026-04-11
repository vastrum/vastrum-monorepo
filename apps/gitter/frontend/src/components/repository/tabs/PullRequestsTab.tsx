import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { type PullRequest, get_repo_pull_requests, get_repo_counts } from '../../../../wasm/pkg';
import EmptyState from '../../common/EmptyState';
import Pagination from '../../common/Pagination';
import NewPullRequestModal from '../../common/NewPullRequestModal';

const ITEMS_PER_PAGE = 20;

interface PullRequestsTabProps {
    repoId: string;
    repoOwner: string;
}

const PullRequestsTab = ({ repoId, repoOwner }: PullRequestsTabProps): React.JSX.Element => {
    const [pullRequests, setPullRequests] = useState<PullRequest[]>([]);
    const [loading, setLoading] = useState(true);
    const [showModal, setShowModal] = useState(false);
    const [currentPage, setCurrentPage] = useState(1);
    const [totalCount, setTotalCount] = useState(0);

    const fetchPullRequests = async (page: number) => {
        try {
            const offset = (page - 1) * ITEMS_PER_PAGE;
            const [data, counts] = await Promise.all([
                get_repo_pull_requests(repoId, ITEMS_PER_PAGE, offset),
                get_repo_counts(repoId),
            ]);
            setPullRequests(data);
            setTotalCount(Number(counts.pr_count));
        } catch (error) {
            console.error('Failed to fetch pull requests:', error);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchPullRequests(currentPage);
    }, [repoId, currentPage]);

    const handleRefresh = () => {
        fetchPullRequests(currentPage);
    };

    const totalPages = Math.ceil(totalCount / ITEMS_PER_PAGE);

    const getStatusColor = (isOpen: boolean): string => {
        return isOpen ? 'fill-app-accent-green' : 'fill-app-accent-purple';
    };

    if (loading) {
        return (
            <div className="lg:col-span-2">
                <div className="text-app-text-secondary p-4">Loading pull requests...</div>
            </div>
        );
    }

    return (
        <div className="lg:col-span-2">
            <div className="bg-app-bg-secondary border border-app-border rounded-lg overflow-hidden">
                {/* Filter/Action Bar */}
                <div className="flex items-center justify-between p-4 border-b border-app-border">
                    <div className="flex items-center gap-2">
                        <button className="text-sm text-app-text-primary hover:text-app-accent-blue">
                            All pull requests
                        </button>
                    </div>
                    <button onClick={() => setShowModal(true)} className="btn-primary text-sm">New pull request</button>
                </div>

                {/* Pull Requests List or Empty State */}
                {pullRequests.length > 0 ? (
                    <div>
                        {pullRequests.map((pr) => (
                            <Link
                                key={pr.id}
                                to={`/repo/${repoId}/pull/${pr.id}`}
                                className="block p-4 border-b border-app-border last:border-b-0 hover:bg-app-hover transition-colors"
                            >
                                <div className="flex items-start gap-3">
                                    <svg viewBox="0 0 16 16" className={`w-4 h-4 ${getStatusColor(pr.is_open)} mt-1 flex-shrink-0`}>
                                        <path d="M1.5 3.25a2.25 2.25 0 1 1 3 2.122v5.256a2.251 2.251 0 1 1-1.5 0V5.372A2.25 2.25 0 0 1 1.5 3.25Zm5.677-.177L9.573.677A.25.25 0 0 1 10 .854V2.5h1A2.5 2.5 0 0 1 13.5 5v5.628a2.251 2.251 0 1 1-1.5 0V5a1 1 0 0 0-1-1h-1v1.646a.25.25 0 0 1-.427.177L7.177 3.427a.25.25 0 0 1 0-.354ZM3.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Zm0 9.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Zm8.25.75a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0Z"></path>
                                    </svg>
                                    <div className="flex-1 min-w-0">
                                        <h4 className="font-medium text-app-text-primary hover:text-app-accent-blue">
                                            {pr.title}
                                        </h4>
                                        <p className="text-sm text-app-text-secondary mt-1 truncate">
                                            #{pr.id} from {pr.head_repo}:{pr.head_branch} • {pr.is_open ? 'Open' : 'Merged'}
                                        </p>
                                    </div>
                                </div>
                            </Link>
                        ))}
                    </div>
                ) : (
                    <EmptyState
                        icon={
                            <svg viewBox="0 0 16 16" className="w-8 h-8 fill-app-text-secondary">
                                <path d="M1.5 3.25a2.25 2.25 0 1 1 3 2.122v5.256a2.251 2.251 0 1 1-1.5 0V5.372A2.25 2.25 0 0 1 1.5 3.25Zm5.677-.177L9.573.677A.25.25 0 0 1 10 .854V2.5h1A2.5 2.5 0 0 1 13.5 5v5.628a2.251 2.251 0 1 1-1.5 0V5a1 1 0 0 0-1-1h-1v1.646a.25.25 0 0 1-.427.177L7.177 3.427a.25.25 0 0 1 0-.354ZM3.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Zm0 9.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Zm8.25.75a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0Z"></path>
                            </svg>
                        }
                        title="No open pull requests"
                    />
                )}
            </div>

            <Pagination currentPage={currentPage} totalPages={totalPages} onPageChange={setCurrentPage} />

            <NewPullRequestModal
                isOpen={showModal}
                onClose={() => setShowModal(false)}
                baseRepository={repoId}
                baseOwner={repoOwner}
                onRefresh={handleRefresh}
            />
        </div>
    );
};

export default PullRequestsTab;
