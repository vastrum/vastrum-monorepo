import React, { useState, useEffect } from 'react';
import { Link, useParams, useNavigate } from 'react-router-dom';
import { type Discussion, type DiscussionReply, reply_to_discussion, get_discussion, get_discussion_replies } from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import MarkdownRenderer from '../components/common/MarkdownRenderer';
import MarkdownEditor from '../components/common/MarkdownEditor';
import Pagination from '../components/common/Pagination';
import { generateAvatarGradient, truncateAddress } from '../utils/avatarGenerator';
import { formatRelativeTime } from '../utils/timeUtils';

const REPLIES_PER_PAGE = 20;

function DiscussionPage(): React.JSX.Element {
    const { repoId, id } = useParams<{ repoId: string; id: string }>();
    const navigate = useNavigate();
    const [commentText, setCommentText] = useState('');
    const [discussion, setDiscussion] = useState<Discussion | null>(null);
    const [replies, setReplies] = useState<DiscussionReply[]>([]);
    const [replyPage, setReplyPage] = useState(1);
    const [loading, setLoading] = useState(true);

    const fetchDiscussion = async (): Promise<void> => {
        if (!repoId || !id) return;
        const foundDiscussion = await get_discussion(repoId, BigInt(id));
        setDiscussion(foundDiscussion ?? null);
        setLoading(false);
    };

    const fetchReplies = async (page: number): Promise<void> => {
        if (!repoId || !id) return;
        const offset = (page - 1) * REPLIES_PER_PAGE;
        const data = await get_discussion_replies(repoId, BigInt(id), REPLIES_PER_PAGE, offset);
        setReplies(data);
    };

    useEffect(() => {
        fetchDiscussion();
    }, [repoId, id]);

    useEffect(() => {
        if (discussion) fetchReplies(replyPage);
    }, [discussion, replyPage]);

    const replyCount = discussion ? Number(discussion.reply_count) : 0;
    const totalReplyPages = Math.ceil(replyCount / REPLIES_PER_PAGE);

    const handleComment = async (): Promise<void> => {
        if (!discussion || !repoId) return;
        const txHash = await reply_to_discussion(commentText, repoId, BigInt(discussion.id));
        setCommentText('');
        await await_tx_inclusion(txHash);
        await fetchDiscussion();
        const lastPage = Math.ceil((replyCount + 1) / REPLIES_PER_PAGE);
        setReplyPage(lastPage);
    };

    if (loading) {
        return <div className="max-w-7xl mx-auto px-5 py-5">Loading...</div>;
    }

    if (!discussion || !repoId) {
        return (
            <div className="max-w-7xl mx-auto px-5 py-5">
                <p>Discussion not found.</p>
                <button onClick={() => navigate(`/repo/${repoId}`)} className="btn-primary mt-4">Back to repository</button>
            </div>
        );
    }

    return (
        <div key={discussion.id} className="max-w-7xl mx-auto px-5 py-5 md:px-6 md:py-6">
            {/* Breadcrumb */}
            <div className="flex items-center gap-2 mb-3 md:mb-4 text-xs md:text-sm overflow-x-auto scrollbar-thin pb-1">
                <Link to={`/repo/${repoId}`} className="text-app-accent-blue hover:underline whitespace-nowrap flex-shrink-0">{repoId}</Link>
                <span className="text-app-text-secondary flex-shrink-0">/</span>
                <Link to={`/repo/${repoId}/discussions`} className="text-app-accent-blue hover:underline whitespace-nowrap flex-shrink-0">Discussions</Link>
                <span className="text-app-text-secondary flex-shrink-0">/</span>
                <span className="text-app-text-secondary whitespace-nowrap flex-shrink-0">#{discussion.id}</span>
            </div>

            {/* Discussion Header */}
            <div className="mb-4 md:mb-6">
                <div className="flex items-start gap-3 md:gap-4 mb-3 md:mb-4">
                    <h1 className="text-xl md:text-2xl lg:text-3xl font-semibold flex-1">
                        {discussion.title} <span className="text-app-text-secondary">#{discussion.id}</span>
                    </h1>
                </div>

                <div className="flex items-center gap-3 flex-wrap">
                    <span className="text-app-text-secondary text-sm">
                        <span className="text-app-text-primary font-medium">{truncateAddress(discussion.from)}</span>
                        {' '}started this discussion {formatRelativeTime(discussion.timestamp)} · {replyCount} comments
                    </span>
                </div>
            </div>

            {/* Main Content */}
            <div className="max-w-4xl">
                {/* Timeline */}
                <div className="space-y-4 md:space-y-6">
                    {/* Initial Discussion */}
                    <div className="flex gap-3 md:gap-4">
                        <div className={`w-8 h-8 md:w-10 md:h-10 rounded-full bg-gradient-to-br ${generateAvatarGradient(discussion.from)} flex-shrink-0`} />
                        <div className="flex-1 min-w-0">
                            <div className="comment-box">
                                <div className="bg-app-bg-tertiary px-3 py-2 md:px-4 md:py-3 border-b border-app-border flex items-center justify-between">
                                    <div className="flex items-center gap-2 text-sm min-w-0">
                                        <strong className="text-app-text-primary font-semibold truncate">{truncateAddress(discussion.from)}</strong>
                                        <span className="text-app-text-secondary flex-shrink-0">{formatRelativeTime(discussion.timestamp)}</span>
                                    </div>
                                </div>
                                <div className="p-3 md:p-4">
                                    {discussion.description ? (
                                        <MarkdownRenderer content={discussion.description} />
                                    ) : (
                                        <span className="text-app-text-secondary">No description provided.</span>
                                    )}
                                </div>
                            </div>
                        </div>
                    </div>

                    {/* Replies */}
                    {replies.map((reply, index) => (
                        <div key={index} className="flex gap-3 md:gap-4">
                            <div className={`w-8 h-8 md:w-10 md:h-10 rounded-full bg-gradient-to-br ${generateAvatarGradient(reply.from)} flex-shrink-0`} />
                            <div className="flex-1 min-w-0">
                                <div className="comment-box">
                                    <div className="bg-app-bg-tertiary px-3 py-2 md:px-4 md:py-3 border-b border-app-border flex items-center justify-between">
                                        <div className="flex items-center gap-2 text-sm min-w-0">
                                            <strong className="text-app-text-primary font-semibold truncate">{truncateAddress(reply.from)}</strong>
                                            <span className="text-app-text-secondary flex-shrink-0">{formatRelativeTime(reply.timestamp)}</span>
                                        </div>
                                    </div>
                                    <div className="p-3 md:p-4">
                                        <MarkdownRenderer content={reply.content} />
                                    </div>
                                </div>
                            </div>
                        </div>
                    ))}

                    <Pagination currentPage={replyPage} totalPages={totalReplyPages} onPageChange={setReplyPage} />

                    {/* Comment Form */}
                    <div className="flex gap-3 md:gap-4">
                        <div className="w-8 h-8 md:w-10 md:h-10 rounded-full bg-gradient-to-br from-app-accent-purple to-app-accent-blue flex-shrink-0" />
                        <div className="flex-1 min-w-0">
                            <div className="comment-box">
                                <div className="bg-app-bg-tertiary px-3 py-2 md:px-4 md:py-3 border-b border-app-border flex gap-4">
                                    <button className="text-sm font-medium text-app-text-primary">Reply</button>
                                </div>
                                <div className="p-3 md:p-4">
                                    <MarkdownEditor
                                        value={commentText}
                                        onChange={setCommentText}
                                        placeholder="Leave a comment"
                                        minHeight="100px"
                                    />
                                </div>
                                <div className="px-3 py-2 md:px-4 md:py-3 border-t border-app-border flex justify-end">
                                    <button className="btn-primary text-sm" onClick={handleComment}>Comment</button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

            </div>
        </div>
    );
}

export default DiscussionPage;
