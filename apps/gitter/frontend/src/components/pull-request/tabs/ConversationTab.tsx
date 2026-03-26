import React, { useState, useEffect } from 'react';
import { GitCommit, Check, CircleAlert, GitMerge, Clock } from 'lucide-react';
import CommentBox from '../../common/CommentBox';
import MarkdownEditor from '../../common/MarkdownEditor';
import Pagination from '../../common/Pagination';
import {
    type PullRequest,
    type PullRequestReply,
    type GetPullRequestDetail,
    type FrontendMergability,
    get_pull_request_replies,
} from '../../../../wasm/pkg';
import { generateAvatarGradient } from '../../../utils/avatarGenerator';
import { formatRelativeTime } from '../../../utils/timeUtils';

const REPLIES_PER_PAGE = 20;

interface ConversationTabProps {
    pr: PullRequest;
    prDetail: GetPullRequestDetail | null;
    repoId: string;
    onComment: (content: string) => Promise<void>;
    onMerge: () => Promise<void>;
}

function ConversationTab({ pr, prDetail, repoId, onComment, onMerge }: ConversationTabProps): React.JSX.Element {
    const [commentText, setCommentText] = useState('');
    const [replies, setReplies] = useState<PullRequestReply[]>([]);
    const [replyPage, setReplyPage] = useState(1);

    const replyCount = Number(pr.reply_count);
    const totalReplyPages = Math.ceil(replyCount / REPLIES_PER_PAGE);

    const fetchReplies = async (page: number): Promise<void> => {
        const offset = (page - 1) * REPLIES_PER_PAGE;
        const data = await get_pull_request_replies(repoId, BigInt(pr.id), REPLIES_PER_PAGE, offset);
        setReplies(data);
    };

    useEffect(() => {
        fetchReplies(replyPage);
    }, [pr.reply_count, replyPage]);

    const handleComment = async (): Promise<void> => {
        if (!commentText.trim()) return;
        await onComment(commentText);
        setCommentText('');
        const lastPage = Math.ceil((replyCount + 1) / REPLIES_PER_PAGE);
        setReplyPage(lastPage);
        fetchReplies(lastPage);
    };

    const getMergeStatusContent = (mergability: FrontendMergability | undefined, isOpen: boolean) => {
        if (!isOpen) {
            return {
                icon: <GitMerge className="w-6 h-6 text-app-accent-purple flex-shrink-0" />,
                title: 'Pull request successfully merged',
                subtitle: 'The changes have been merged into the base branch.',
                showButton: false
            };
        }

        switch (mergability) {
            case 'CanMerge':
                return {
                    icon: <Check className="w-6 h-6 text-app-accent-green flex-shrink-0" />,
                    title: 'This branch has no conflicts with the base branch',
                    subtitle: 'Merging can be performed automatically by the owner of this repository.',
                    showButton: true,
                    canMerge: true
                };
            case 'CannotMergeConflict':
                return {
                    icon: <CircleAlert className="w-6 h-6 text-app-accent-red flex-shrink-0" />,
                    title: 'This branch has conflicts that must be resolved',
                    subtitle: 'Resolve conflicts before merging.',
                    showButton: true,
                    canMerge: false
                };
            case 'AlreadyUpToDate':
                return {
                    icon: <Check className="w-6 h-6 text-app-accent-blue flex-shrink-0" />,
                    title: 'Already up to date',
                    subtitle: 'The base branch already contains these changes.',
                    showButton: false
                };
            case 'AlreadyMerged':
                return {
                    icon: <GitMerge className="w-6 h-6 text-app-accent-purple flex-shrink-0" />,
                    title: 'Already merged',
                    subtitle: 'These changes have already been merged.',
                    showButton: false
                };
            default:
                return {
                    icon: <Clock className="w-6 h-6 text-app-text-secondary flex-shrink-0" />,
                    title: 'Loading merge status...',
                    subtitle: 'Checking if this branch can be merged.',
                    showButton: false
                };
        }
    };

    const mergeStatus = getMergeStatusContent(prDetail?.frontend_mergability, pr.is_open);
    const commits = prDetail?.commits_to_merge || [];

    return (
        <div className="lg:col-span-2 space-y-5 md:space-y-6">
            {/* Merge Status Box */}
            <div className="bg-app-bg-secondary border border-app-border rounded-lg p-4 md:p-5">
                <div className={`flex items-center gap-3 md:gap-4 ${mergeStatus.showButton ? 'mb-4' : ''}`}>
                    {mergeStatus.icon}
                    <div className="flex-1">
                        <div className="font-semibold text-base">{mergeStatus.title}</div>
                        <div className="text-app-text-secondary text-sm">{mergeStatus.subtitle}</div>
                    </div>
                </div>
                {mergeStatus.showButton && prDetail?.is_owner && (
                    <div className="flex gap-2">
                        <button
                            className={mergeStatus.canMerge ? "btn-primary flex items-center gap-2" : "btn-secondary flex items-center gap-2 opacity-50 cursor-not-allowed"}
                            disabled={!mergeStatus.canMerge}
                            onClick={onMerge}
                        >
                            Merge pull request
                        </button>
                    </div>
                )}
            </div>

            {/* Initial Description */}
            <div className="flex gap-4">
                <div className={`w-8 h-8 md:w-10 md:h-10 rounded-full bg-gradient-to-br ${generateAvatarGradient(pr.from)} flex-shrink-0`} />
                <div className="flex-1">
                    <CommentBox
                        author={pr.from}
                        content={pr.description || 'No description provided.'}
                        timeAgo=""
                        badge="Author"
                    />
                </div>
            </div>

            {/* Commits Event */}
            {commits.length > 0 && (
                <div className="flex gap-4">
                    <div className="w-8 h-8 md:w-10 md:h-10 rounded-full bg-app-bg-tertiary border border-app-border flex items-center justify-center flex-shrink-0">
                        <GitCommit className="w-5 h-5 text-app-text-secondary" />
                    </div>
                    <div className="flex-1 pt-2">
                        <div className="text-sm text-app-text-secondary mb-3">
                            Added {commits.length} {commits.length === 1 ? 'commit' : 'commits'}
                        </div>
                        <div className="bg-app-bg-secondary border border-app-border rounded-lg overflow-hidden">
                            {commits.map((commit, index) => (
                                <div
                                    key={index}
                                    className="flex items-center gap-2 md:gap-3 px-3 py-2 md:px-4 md:py-3 border-b border-app-border last:border-b-0"
                                >
                                    <div className={`w-5 h-5 md:w-6 md:h-6 rounded-full bg-gradient-to-br ${generateAvatarGradient(commit.author_name)} flex-shrink-0`} />
                                    <span className="flex-1 text-sm text-app-text-primary truncate">{commit.message}</span>
                                </div>
                            ))}
                        </div>
                    </div>
                </div>
            )}

            {/* Replies/Comments */}
            {replies.map((reply, index) => (
                <div key={index} className="flex gap-4">
                    <div className={`w-8 h-8 md:w-10 md:h-10 rounded-full bg-gradient-to-br ${generateAvatarGradient(reply.from)} flex-shrink-0`} />
                    <div className="flex-1">
                        <CommentBox
                            author={reply.from}
                            content={reply.content}
                            timeAgo={formatRelativeTime(reply.timestamp)}
                            badge={reply.from === pr.from ? 'Author' : undefined}
                        />
                    </div>
                </div>
            ))}

            <Pagination currentPage={replyPage} totalPages={totalReplyPages} onPageChange={setReplyPage} />

            {/* Comment Form */}
            <div className="flex gap-4">
                <div className="w-8 h-8 md:w-10 md:h-10 rounded-full bg-gradient-to-br from-app-accent-purple to-app-accent-blue flex-shrink-0" />
                <div className="flex-1">
                    <div className="comment-box">
                        <div className="bg-app-bg-tertiary px-4 py-3 md:px-5 md:py-4 border-b border-app-border flex gap-4">
                            <button className="text-sm font-medium text-app-text-primary">Reply</button>
                        </div>
                        <div className="p-4 md:p-5">
                            <MarkdownEditor
                                value={commentText}
                                onChange={setCommentText}
                                placeholder="Leave a comment"
                                minHeight="120px"
                            />
                        </div>
                        <div className="px-4 py-3 md:px-5 md:py-4 border-t border-app-border flex justify-end">
                            <button className="btn-primary text-sm" onClick={handleComment}>Comment</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}

export default ConversationTab;
