import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { type Issue, get_repo_issues, get_repo_counts } from '../../../../wasm/pkg';
import EmptyState from '../../common/EmptyState';
import Pagination from '../../common/Pagination';
import NewIssueModal from '../modals/NewIssueModal';
import { formatRelativeTime } from '../../../utils/timeUtils';
import { truncateAddress } from '../../../utils/avatarGenerator';

const ITEMS_PER_PAGE = 20;

interface IssuesTabProps {
    repoId: string;
}

const IssuesTab = ({ repoId }: IssuesTabProps): React.JSX.Element => {
    const [showModal, setShowModal] = useState(false);
    const [issues, setIssues] = useState<Issue[]>([]);
    const [loading, setLoading] = useState(true);
    const [currentPage, setCurrentPage] = useState(1);
    const [totalCount, setTotalCount] = useState(0);

    const fetchIssues = async (page: number) => {
        try {
            const offset = (page - 1) * ITEMS_PER_PAGE;
            const [data, counts] = await Promise.all([
                get_repo_issues(repoId, ITEMS_PER_PAGE, offset),
                get_repo_counts(repoId),
            ]);
            setIssues(data);
            setTotalCount(Number(counts.issue_count));
        } catch (error) {
            console.error('Failed to fetch issues:', error);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchIssues(currentPage);
    }, [repoId, currentPage]);

    const handleRefresh = () => {
        fetchIssues(currentPage);
    };

    const totalPages = Math.ceil(totalCount / ITEMS_PER_PAGE);

    if (loading) {
        return (
            <div className="lg:col-span-2">
                <div className="text-app-text-secondary p-4">Loading issues...</div>
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
                            All issues
                        </button>
                    </div>
                    <button onClick={() => setShowModal(true)} className="btn-primary text-sm">New issue</button>
                </div>

                {/* Issues List or Empty State */}
                {issues.length > 0 ? (
                    <div>
                        {issues.map((issue) => (
                            <Link
                                key={issue.id}
                                to={`/repo/${repoId}/issue/${issue.id}`}
                                className="block p-4 border-b border-app-border last:border-b-0 hover:bg-app-hover transition-colors"
                            >
                                <div className="flex items-start gap-3">
                                    <svg viewBox="0 0 16 16" className="w-4 h-4 fill-app-accent-green mt-1 flex-shrink-0">
                                        <path d="M8 9.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3Z"></path>
                                        <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Z"></path>
                                    </svg>
                                    <div className="flex-1 min-w-0">
                                        <h4 className="font-medium text-app-text-primary hover:text-app-accent-blue">
                                            {issue.title}
                                        </h4>
                                        <p className="text-sm text-app-text-secondary mt-1">
                                            #{issue.id} opened {formatRelativeTime(issue.timestamp)} by {truncateAddress(issue.from)}
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
                                <path d="M8 9.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3Z"></path>
                                <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Z"></path>
                            </svg>
                        }
                        title="No open issues"
                    />
                )}
            </div>

            <Pagination currentPage={currentPage} totalPages={totalPages} onPageChange={setCurrentPage} />

            <NewIssueModal isOpen={showModal} onClose={() => setShowModal(false)} repoId={repoId} onRefresh={handleRefresh} />
        </div>
    );
};

export default IssuesTab;
