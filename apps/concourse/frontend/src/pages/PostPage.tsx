import React, { useState, useEffect, useRef } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { type JSPost, type JSPostReply, get_post, get_post_replies, reply_to_post, delete_post, delete_reply, get_moderators, get_my_public_key } from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import MarkdownRenderer from '../components/common/MarkdownRenderer';
import Breadcrumbs from '../components/forum/Breadcrumbs';
import Pagination from '../components/forum/Pagination';
import QuoteCard, { extractQuote } from '../components/forum/QuoteCard';
import PostItem from '../components/forum/PostItem';

const REPLIES_PER_PAGE = 20;

function PostPage(): React.JSX.Element {
    const { category, id } = useParams<{ category: string; id: string }>();
    const categoryName = decodeURIComponent(category || '');
    const navigate = useNavigate();
    const [post, setPost] = useState<JSPost | null>(null);
    const [replies, setReplies] = useState<JSPostReply[]>([]);
    const [replyPage, setReplyPage] = useState(1);
    const [loading, setLoading] = useState(true);
    const [replyText, setReplyText] = useState('');
    const [submitting, setSubmitting] = useState(false);
    const [showReplyBox, setShowReplyBox] = useState(false);
    const [showReplyPreview, setShowReplyPreview] = useState(false);
    const [quoteData, setQuoteData] = useState<{ author: string; content: string } | null>(null);
    const [isModerator, setIsModerator] = useState(false);
    const [confirmDelete, setConfirmDelete] = useState<number | null>(null);
    const replyTextareaRef = useRef<HTMLTextAreaElement>(null);

    const postId = BigInt(Number(id));
    const totalReplyPages = post ? Math.max(1, Math.ceil(Number(post.reply_count) / REPLIES_PER_PAGE)) : 1;

    const fetchPost = async (): Promise<JSPost | null> => {
        const foundPost = await get_post(categoryName, postId);
        setPost(foundPost ?? null);
        setLoading(false);
        return foundPost ?? null;
    };

    const fetchReplies = async (page: number): Promise<void> => {
        const offset = (page - 1) * REPLIES_PER_PAGE;
        const fetched = await get_post_replies(categoryName, postId, REPLIES_PER_PAGE, offset);
        setReplies(fetched);
    };

    useEffect(() => {
        fetchPost();
    }, [id, categoryName]);

    useEffect(() => {
        if (post) {
            fetchReplies(replyPage);
        }
    }, [post, replyPage]);

    useEffect(() => {
        (async () => {
            const [mods, myKey] = await Promise.all([get_moderators(), get_my_public_key()]);
            setIsModerator(mods.includes(myKey));
        })();
    }, []);

    const handleQuote = (author: string, content: string): void => {
        const existing = extractQuote(content);
        const rawContent = existing ? existing.rest : content;
        const firstLine = rawContent.split('\n')[0].slice(0, 120);
        setQuoteData({ author, content: firstLine });
        setShowReplyBox(true);
        setShowReplyPreview(false);
        setTimeout(() => replyTextareaRef.current?.focus(), 50);
    };

    const handleDeletePost = async () => {
        if (!post) return;
        if (confirmDelete !== -1) { setConfirmDelete(-1); return; }
        setConfirmDelete(null);
        await delete_post(categoryName, BigInt(post.id));
        navigate(`/category/${encodeURIComponent(categoryName)}`);
    };

    const handleDeleteReply = async (replyId: number) => {
        if (!post) return;
        if (confirmDelete !== replyId) { setConfirmDelete(replyId); return; }
        setConfirmDelete(null);
        const txHash = await delete_reply(categoryName, BigInt(post.id), BigInt(replyId));
        await await_tx_inclusion(txHash);
        fetchPost();
    };

    const handleReply = async (): Promise<void> => {
        if (!post || !replyText.trim() || submitting) return;
        setSubmitting(true);
        let finalText = replyText;
        if (quoteData) {
            finalText = `<!--quote:${quoteData.author}:${quoteData.content}-->\n${replyText}`;
        }
        const txHash = await reply_to_post(categoryName, BigInt(post.id), finalText);
        setReplyText('');
        setQuoteData(null);
        setSubmitting(false);
        setShowReplyBox(false);
        setShowReplyPreview(false);
        await await_tx_inclusion(txHash);
        const updated = await fetchPost();
        if (updated) {
            const lastPage = Math.max(1, Math.ceil(Number(updated.reply_count) / REPLIES_PER_PAGE));
            setReplyPage(lastPage);
        }
    };

    if (loading) {
        return (
            <main style={{ maxWidth: 1200, margin: '0 auto', padding: '40px 20px' }}>
                <div style={{ textAlign: 'center', color: '#919191' }}>Loading...</div>
            </main>
        );
    }

    if (!post) {
        return (
            <main style={{ maxWidth: 1200, margin: '0 auto', padding: '40px 20px', textAlign: 'center' }}>
                <p style={{ color: '#919191', marginBottom: 16 }}>Topic not found</p>
                <button
                    onClick={() => navigate(`/category/${encodeURIComponent(categoryName)}`)}
                    style={{
                        background: 'none',
                        border: 'none',
                        color: '#08c',
                        cursor: 'pointer',
                        fontSize: 14,
                    }}
                >
                    Back to {categoryName}
                </button>
            </main>
        );
    }

    return (
        <main style={{ maxWidth: 1100, margin: '0 auto', padding: '0 20px 20px' }}>
            <Breadcrumbs items={[
                { label: 'Concourse', to: '/' },
                { label: categoryName, to: `/category/${encodeURIComponent(categoryName)}` },
            ]} />

            <div style={{
                backgroundColor: '#fff',
                borderRadius: 8,
                border: '1px solid #e1e3e5',
                padding: '0 20px 20px',
            }}>
                {/* Topic title */}
                <h1 style={{
                    fontSize: 24,
                    fontWeight: 400,
                    color: '#222',
                    margin: '0',
                    padding: '20px 0',
                    lineHeight: 1.3,
                }}>
                    {post.title}
                </h1>

                {/* Posts */}
                <div style={{ borderTop: '2px solid #e9e9e9' }}>
                    {/* Original post */}
                    <PostItem
                        author={post.from}
                        content={post.content}
                        timestamp={post.timestamp}
                        index={0}
                        onQuote={() => handleQuote(post.from, post.content)}
                        onDelete={isModerator ? handleDeletePost : undefined}
                        deleteConfirming={confirmDelete === -1}
                    />

                    {/* Replies */}
                    {replies.map((reply, index) => (
                        <PostItem
                            key={reply.id}
                            author={reply.from}
                            content={reply.content}
                            timestamp={reply.timestamp}
                            index={(replyPage - 1) * REPLIES_PER_PAGE + index + 1}
                            onQuote={() => handleQuote(reply.from, reply.content)}
                            onDelete={isModerator ? () => handleDeleteReply(Number(reply.id)) : undefined}
                            deleteConfirming={confirmDelete === Number(reply.id)}
                        />
                    ))}
                </div>

                <Pagination
                    currentPage={replyPage}
                    totalPages={totalReplyPages}
                    onPageChange={setReplyPage}
                />

                {/* Reply button or box */}
                <div style={{ padding: '20px 0 0' }}>
                {!showReplyBox ? (
                    <button
                        onClick={() => setShowReplyBox(true)}
                        className="btn"
                        style={{
                            backgroundColor: '#08c',
                            color: 'white',
                            border: 'none',
                            borderRadius: 4,
                            padding: '10px 16px',
                            fontSize: 14,
                            fontWeight: 500,
                            cursor: 'pointer',
                        }}
                    >
                        Reply
                    </button>
                ) : (
                    <div style={{
                        border: '1px solid #e9e9e9',
                        borderRadius: 4,
                        overflow: 'hidden',
                    }}>
                        <div style={{
                            display: 'flex',
                            gap: 8,
                            padding: '8px 12px',
                            borderBottom: '1px solid #e9e9e9',
                            backgroundColor: '#fafafa',
                        }}>
                            <button
                                type="button"
                                onClick={() => setShowReplyPreview(false)}
                                style={{
                                    background: !showReplyPreview ? '#e9e9e9' : 'none',
                                    border: 'none',
                                    color: !showReplyPreview ? '#333' : '#919191',
                                    fontSize: 13,
                                    fontWeight: 500,
                                    cursor: 'pointer',
                                    padding: '4px 10px',
                                    borderRadius: 4,
                                }}
                            >
                                Write
                            </button>
                            <button
                                type="button"
                                onClick={() => setShowReplyPreview(true)}
                                style={{
                                    background: showReplyPreview ? '#e9e9e9' : 'none',
                                    border: 'none',
                                    color: showReplyPreview ? '#333' : '#919191',
                                    fontSize: 13,
                                    fontWeight: 500,
                                    cursor: 'pointer',
                                    padding: '4px 10px',
                                    borderRadius: 4,
                                }}
                            >
                                Preview
                            </button>
                        </div>
                        {!showReplyPreview ? (
                            <div>
                                {quoteData && (
                                    <div style={{ padding: '8px 12px 0' }}>
                                        <QuoteCard
                                            author={quoteData.author}
                                            quoted={quoteData.content}
                                            onDismiss={() => setQuoteData(null)}
                                        />
                                    </div>
                                )}
                                <textarea
                                    ref={replyTextareaRef}
                                    className="input-field"
                                    style={{
                                        border: 'none',
                                        borderRadius: 0,
                                        minHeight: 150,
                                        resize: 'vertical',
                                    }}
                                    placeholder="Type your reply... (Markdown supported)"
                                    value={replyText}
                                    onChange={(e) => setReplyText(e.target.value)}
                                    autoFocus
                                />
                            </div>
                        ) : (
                            <div style={{
                                minHeight: 150,
                                padding: '8px 12px',
                                backgroundColor: '#fff',
                            }}>
                                {quoteData && (
                                    <QuoteCard
                                        author={quoteData.author}
                                        quoted={quoteData.content}
                                    />
                                )}
                                {replyText.trim() ? (
                                    <MarkdownRenderer content={replyText} />
                                ) : (
                                    <span style={{ color: '#919191', fontSize: 14 }}>
                                        Nothing to preview
                                    </span>
                                )}
                            </div>
                        )}
                        <div style={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                            padding: '12px 16px',
                            borderTop: '1px solid #e9e9e9',
                            backgroundColor: '#f8f8f8',
                        }}>
                            <button
                                onClick={() => {
                                    setShowReplyBox(false);
                                    setReplyText('');
                                    setQuoteData(null);
                                    setShowReplyPreview(false);
                                }}
                                style={{
                                    background: 'none',
                                    border: 'none',
                                    color: '#919191',
                                    fontSize: 14,
                                    cursor: 'pointer',
                                }}
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleReply}
                                disabled={!replyText.trim() || submitting}
                                className="btn"
                                style={{
                                    backgroundColor: '#08c',
                                    color: 'white',
                                    border: 'none',
                                    borderRadius: 4,
                                    padding: '8px 14px',
                                    fontSize: 14,
                                    fontWeight: 500,
                                    cursor: 'pointer',
                                    opacity: (!replyText.trim() || submitting) ? 0.5 : 1,
                                }}
                            >
                                {submitting ? 'Posting...' : 'Reply'}
                            </button>
                        </div>
                    </div>
                )}
            </div>
            </div>
        </main>
    );
}

export default PostPage;
