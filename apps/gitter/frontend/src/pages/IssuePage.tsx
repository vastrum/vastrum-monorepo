import React, { useState, useEffect } from 'react';
import { Link, useParams, useNavigate } from 'react-router-dom';
import { type Issue, type IssueReply, reply_to_issue, get_issue, get_issue_replies } from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import MarkdownRenderer from '../components/common/MarkdownRenderer';
import MarkdownEditor from '../components/common/MarkdownEditor';
import Pagination from '../components/common/Pagination';
import { generateAvatarGradient, truncateAddress } from '../utils/avatarGenerator';
import { formatRelativeTime } from '../utils/timeUtils';

const REPLIES_PER_PAGE = 20;

function IssuePage(): React.JSX.Element {
    const { repoId, id } = useParams<{ repoId: string; id: string }>();
    const navigate = useNavigate();
    const [commentText, setCommentText] = useState('');
    const [issue, setIssue] = useState<Issue | null>(null);
    const [replies, setReplies] = useState<IssueReply[]>([]);
    const [replyPage, setReplyPage] = useState(1);
    const [loading, setLoading] = useState(true);

    const fetchIssue = async (): Promise<void> => {
        if (!repoId || !id) return;
        const foundIssue = await get_issue(repoId, BigInt(id));
        setIssue(foundIssue ?? null);
        setLoading(false);
    };

    const fetchReplies = async (page: number): Promise<void> => {
        if (!repoId || !id) return;
        const offset = (page - 1) * REPLIES_PER_PAGE;
        const data = await get_issue_replies(repoId, BigInt(id), REPLIES_PER_PAGE, offset);
        setReplies(data);
    };

    useEffect(() => {
        fetchIssue();
    }, [repoId, id]);

    useEffect(() => {
        if (issue) fetchReplies(replyPage);
    }, [issue, replyPage]);

    const replyCount = issue ? Number(issue.reply_count) : 0;
    const totalReplyPages = Math.ceil(replyCount / REPLIES_PER_PAGE);

    const handleComment = async (): Promise<void> => {
        if (!issue || !repoId) return;
        const txHash = await reply_to_issue(commentText, repoId, BigInt(issue.id));
        setCommentText('');
        await await_tx_inclusion(txHash);
        await fetchIssue();
        const lastPage = Math.ceil((replyCount + 1) / REPLIES_PER_PAGE);
        setReplyPage(lastPage);
    };

    if (loading) {
        return <div className="max-w-7xl mx-auto px-5 py-5">Loading...</div>;
    }

    if (!issue || !repoId) {
        return (
            <div className="max-w-7xl mx-auto px-5 py-5">
                <p>Issue not found.</p>
                <button onClick={() => navigate(`/repo/${repoId}`)} className="btn-primary mt-4">Back to repository</button>
            </div>
        );
    }

    return (
        <div key={issue.id} className="max-w-7xl mx-auto px-5 py-5 md:px-6 md:py-6">
            {/* Breadcrumb */}
            <div className="flex items-center gap-2 mb-3 md:mb-4 text-xs md:text-sm overflow-x-auto scrollbar-thin pb-1">
                <Link to={`/repo/${repoId}`} className="text-app-accent-blue hover:underline whitespace-nowrap flex-shrink-0">{repoId}</Link>
                <span className="text-app-text-secondary flex-shrink-0">/</span>
                <Link to={`/repo/${repoId}/issues`} className="text-app-accent-blue hover:underline whitespace-nowrap flex-shrink-0">Issues</Link>
                <span className="text-app-text-secondary flex-shrink-0">/</span>
                <span className="text-app-text-secondary whitespace-nowrap flex-shrink-0">#{issue.id}</span>
            </div>

            {/* Issue Header */}
            <div className="mb-4 md:mb-6">
                <div className="flex items-start gap-3 md:gap-4 mb-3 md:mb-4">
                    <h1 className="text-xl md:text-2xl lg:text-3xl font-semibold flex-1">
                        {issue.title} <span className="text-app-text-secondary">#{issue.id}</span>
                    </h1>
                </div>

                <div className="flex items-center gap-3 flex-wrap">
                    <span className="inline-flex items-center gap-2 bg-app-accent-green text-white px-3 py-1.5 rounded-full text-sm font-semibold">
                        <svg viewBox="0 0 16 16" className="w-4 h-4 fill-current">
                            <path d="M8 9.5a1.5 1.5 0 1 0 0-3 1.5 1.5 0 0 0 0 3Z"></path>
                            <path d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Z"></path>
                        </svg>
                        Open
                    </span>
                    <span className="text-app-text-secondary text-sm">
                        <span className="text-app-text-primary font-medium">{truncateAddress(issue.from)}</span>
                        {' '}opened this issue {formatRelativeTime(issue.timestamp)} · {replyCount} comments
                    </span>
                </div>
            </div>

            {/* Main Content */}
            <div className="max-w-4xl">
                {/* Timeline */}
                <div className="space-y-4 md:space-y-6">
                    {/* Initial Issue */}
                    <div className="flex gap-3 md:gap-4">
                        <div className={`w-8 h-8 md:w-10 md:h-10 rounded-full bg-gradient-to-br ${generateAvatarGradient(issue.from)} flex-shrink-0`} />
                        <div className="flex-1 min-w-0">
                            <div className="comment-box">
                                <div className="bg-app-bg-tertiary px-3 py-2 md:px-4 md:py-3 border-b border-app-border flex items-center justify-between">
                                    <div className="flex items-center gap-2 text-sm min-w-0">
                                        <strong className="text-app-text-primary font-semibold truncate">{truncateAddress(issue.from)}</strong>
                                        <span className="text-app-text-secondary flex-shrink-0">{formatRelativeTime(issue.timestamp)}</span>
                                    </div>
                                </div>
                                <div className="p-3 md:p-4">
                                    {issue.description ? (
                                        <MarkdownRenderer content={issue.description} />
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

export default IssuePage;
