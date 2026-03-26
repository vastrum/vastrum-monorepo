import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { type Discussion, get_repo_discussions, get_repo_counts } from '../../../../wasm/pkg';
import EmptyState from '../../common/EmptyState';
import Pagination from '../../common/Pagination';
import NewDiscussionModal from '../modals/NewDiscussionModal';
import { formatRelativeTime } from '../../../utils/timeUtils';
import { truncateAddress } from '../../../utils/avatarGenerator';

const ITEMS_PER_PAGE = 20;

interface DiscussionsTabProps {
    repoId: string;
}

const DiscussionsTab = ({ repoId }: DiscussionsTabProps): React.JSX.Element => {
    const [showModal, setShowModal] = useState(false);
    const [discussions, setDiscussions] = useState<Discussion[]>([]);
    const [loading, setLoading] = useState(true);
    const [currentPage, setCurrentPage] = useState(1);
    const [totalCount, setTotalCount] = useState(0);

    const fetchDiscussions = async (page: number) => {
        try {
            const offset = (page - 1) * ITEMS_PER_PAGE;
            const [data, counts] = await Promise.all([
                get_repo_discussions(repoId, ITEMS_PER_PAGE, offset),
                get_repo_counts(repoId),
            ]);
            setDiscussions(data);
            setTotalCount(Number(counts.discussion_count));
        } catch (error) {
            console.error('Failed to fetch discussions:', error);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchDiscussions(currentPage);
    }, [repoId, currentPage]);

    const handleRefresh = () => {
        fetchDiscussions(currentPage);
    };

    const totalPages = Math.ceil(totalCount / ITEMS_PER_PAGE);

    if (loading) {
        return (
            <div className="lg:col-span-2">
                <div className="text-app-text-secondary p-4">Loading discussions...</div>
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
                            All discussions
                        </button>
                    </div>
                    <button onClick={() => setShowModal(true)} className="btn-primary text-sm">New discussion</button>
                </div>

                {/* Discussions List or Empty State */}
                {discussions.length > 0 ? (
                    <div>
                        {discussions.map((discussion) => (
                            <Link
                                key={discussion.id}
                                to={`/repo/${repoId}/discussion/${discussion.id}`}
                                className="block p-4 border-b border-app-border last:border-b-0 hover:bg-app-hover transition-colors"
                            >
                                <div className="flex items-start gap-3">
                                    <svg viewBox="0 0 16 16" className="w-4 h-4 fill-app-text-secondary mt-1 flex-shrink-0">
                                        <path d="M1.75 1h8.5c.966 0 1.75.784 1.75 1.75v5.5A1.75 1.75 0 0 1 10.25 10H7.061l-2.574 2.573A1.458 1.458 0 0 1 2 11.543V10h-.25A1.75 1.75 0 0 1 0 8.25v-5.5C0 1.784.784 1 1.75 1Z"></path>
                                    </svg>
                                    <div className="flex-1 min-w-0">
                                        <h4 className="font-medium text-app-text-primary hover:text-app-accent-blue">
                                            {discussion.title}
                                        </h4>
                                        <p className="text-sm text-app-text-secondary mt-1">
                                            {Number(discussion.reply_count)} comments • Started by {truncateAddress(discussion.from)} {formatRelativeTime(discussion.timestamp)}
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
                                <path d="M1.75 1h8.5c.966 0 1.75.784 1.75 1.75v5.5A1.75 1.75 0 0 1 10.25 10H7.061l-2.574 2.573A1.458 1.458 0 0 1 2 11.543V10h-.25A1.75 1.75 0 0 1 0 8.25v-5.5C0 1.784.784 1 1.75 1ZM1.5 2.75v5.5c0 .138.112.25.25.25h1a.75.75 0 0 1 .75.75v2.19l2.72-2.72a.749.749 0 0 1 .53-.22h3.5a.25.25 0 0 0 .25-.25v-5.5a.25.25 0 0 0-.25-.25h-8.5a.25.25 0 0 0-.25.25Zm13 2a.25.25 0 0 0-.25-.25h-.5a.75.75 0 0 1 0-1.5h.5c.966 0 1.75.784 1.75 1.75v5.5A1.75 1.75 0 0 1 14.25 12H14v1.543a1.458 1.458 0 0 1-2.487 1.03L9.22 12.28a.749.749 0 0 1 .326-1.275.749.749 0 0 1 .734.215l2.22 2.22v-2.19a.75.75 0 0 1 .75-.75h1a.25.25 0 0 0 .25-.25Z"></path>
                            </svg>
                        }
                        title="No discussions yet"
                    />
                )}
            </div>

            <Pagination currentPage={currentPage} totalPages={totalPages} onPageChange={setCurrentPage} />

            <NewDiscussionModal isOpen={showModal} onClose={() => setShowModal(false)} repoId={repoId} onRefresh={handleRefresh} />
        </div>
    );
};

export default DiscussionsTab;
