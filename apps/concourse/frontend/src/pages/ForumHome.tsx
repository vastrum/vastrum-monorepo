import React, { useState, useEffect } from 'react';
import { Link, useParams } from 'react-router-dom';
import { type JSPost, get_category_posts, get_category_post_count } from '../../wasm/pkg';
import { truncateAddress } from '../utils/avatarGenerator';
import { formatRelativeTime } from '../utils/timeUtils';
import NewPostModal from '../components/forum/NewPostModal';
import Breadcrumbs from '../components/forum/Breadcrumbs';
import Pagination from '../components/forum/Pagination';
import { stringToColor } from '../utils/colorUtils';

const POSTS_PER_PAGE = 20;

function ForumHome(): React.JSX.Element {
    const { category } = useParams<{ category: string }>();
    const categoryName = decodeURIComponent(category || '');
    const [posts, setPosts] = useState<JSPost[]>([]);
    const [loading, setLoading] = useState(true);
    const [showNewPostModal, setShowNewPostModal] = useState(false);
    const [currentPage, setCurrentPage] = useState(1);
    const [totalPosts, setTotalPosts] = useState(0);

    const totalPages = Math.max(1, Math.ceil(totalPosts / POSTS_PER_PAGE));

    const fetchPosts = async (page: number): Promise<void> => {
        setLoading(true);
        const offset = (page - 1) * POSTS_PER_PAGE;
        const [fetchedPosts, count] = await Promise.all([
            get_category_posts(categoryName, POSTS_PER_PAGE, offset),
            get_category_post_count(categoryName),
        ]);
        setPosts(fetchedPosts);
        setTotalPosts(Number(count));
        setLoading(false);
    };

    useEffect(() => {
        setCurrentPage(1);
        fetchPosts(1);
    }, [categoryName]);

    const handlePageChange = (page: number): void => {
        setCurrentPage(page);
        fetchPosts(page);
    };

    const handleRefresh = (): void => {
        fetchPosts(currentPage);
    };

    if (loading) {
        return (
            <main style={{ maxWidth: 1200, margin: '0 auto', padding: '40px 20px' }}>
                <div style={{ textAlign: 'center', color: '#919191' }}>Loading...</div>
            </main>
        );
    }

    return (
        <main style={{ maxWidth: 1200, margin: '0 auto', padding: '0 20px 20px' }}>
            <Breadcrumbs items={[
                { label: 'Concourse', to: '/' },
                { label: categoryName },
            ]} />

            {/* Category header + New Topic */}
            <div style={{
                display: 'flex',
                justifyContent: 'space-between',
                alignItems: 'center',
                padding: '0 0 16px',
            }}>
                <h2 style={{ fontSize: 20, fontWeight: 500, color: '#222', margin: 0 }}>
                    {categoryName}
                </h2>
                <button
                    onClick={() => setShowNewPostModal(true)}
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
                    }}
                >
                    + New Topic
                </button>
            </div>

            <div style={{
                backgroundColor: '#fff',
                borderRadius: 8,
                border: '1px solid #e1e3e5',
                padding: '0 20px',
            }}>
                {/* Table header */}
                <div style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    padding: '12px 0',
                    borderBottom: '2px solid #e9e9e9',
                    color: '#919191',
                    fontSize: 13,
                }}>
                    <div style={{ fontWeight: 500 }}>Topic</div>
                    <div style={{
                        display: 'flex',
                        gap: 40,
                        minWidth: 160,
                        justifyContent: 'flex-end',
                    }}>
                        <span style={{ width: 60, textAlign: 'center' }}>Replies</span>
                        <span style={{ width: 60, textAlign: 'center' }}>Activity</span>
                    </div>
                </div>

                {/* Topics list */}
                <div>
                    {posts.length === 0 ? (
                        <div style={{
                            padding: '40px 0',
                            textAlign: 'center',
                            color: '#919191',
                        }}>
                            No topics yet. Start a discussion!
                        </div>
                    ) : (
                        posts.map((post) => {
                            const replyCount = Number(post.reply_count);
                            const lastActivityTime = post.last_bump_time || post.timestamp;

                            return (
                                <div
                                    key={post.id}
                                    className="topic-row"
                                    style={{
                                        display: 'flex',
                                        justifyContent: 'space-between',
                                        alignItems: 'center',
                                        padding: '14px 0',
                                        borderBottom: '1px solid #e9e9e9',
                                    }}
                                >
                                    <div style={{
                                        display: 'flex',
                                        alignItems: 'center',
                                        gap: 16,
                                        flex: 1,
                                        minWidth: 0,
                                    }}>
                                        <div style={{ flex: 1, minWidth: 0 }}>
                                            {/* Title */}
                                            <div style={{
                                                display: 'flex',
                                                alignItems: 'center',
                                                marginBottom: 6,
                                            }}>
                                                <Link
                                                    to={`/category/${encodeURIComponent(categoryName)}/topic/${post.id}`}
                                                    className="topic-title"
                                                    style={{
                                                        fontSize: 16,
                                                        color: '#222',
                                                        textDecoration: 'none',
                                                        fontWeight: 400,
                                                        lineHeight: 1.3,
                                                        transition: 'color 0.15s',
                                                    }}
                                                >
                                                    {post.title}
                                                </Link>
                                            </div>

                                            {/* Author */}
                                            <div style={{
                                                display: 'flex',
                                                alignItems: 'center',
                                                gap: 6,
                                            }}>
                                                <span style={{
                                                    fontSize: 11,
                                                    color: '#919191',
                                                }}>
                                                    {truncateAddress(post.from)}
                                                </span>
                                            </div>
                                        </div>

                                        {/* OP Avatar */}
                                        <div style={{
                                            display: 'flex',
                                            alignItems: 'center',
                                            marginLeft: 'auto',
                                            paddingRight: 20,
                                        }}>
                                            <div
                                                style={{
                                                    width: 28,
                                                    height: 28,
                                                    borderRadius: '50%',
                                                    border: '2px solid #fff',
                                                    display: 'flex',
                                                    alignItems: 'center',
                                                    justifyContent: 'center',
                                                    backgroundColor: stringToColor(post.from),
                                                }}
                                            >
                                                <span style={{
                                                    fontSize: 11,
                                                    fontWeight: 600,
                                                    color: '#fff',
                                                }}>
                                                    {post.from.slice(0, 2).toUpperCase()}
                                                </span>
                                            </div>
                                        </div>
                                    </div>

                                    {/* Stats */}
                                    <div style={{
                                        display: 'flex',
                                        gap: 40,
                                        minWidth: 160,
                                        justifyContent: 'flex-end',
                                    }}>
                                        <span style={{
                                            width: 60,
                                            textAlign: 'center',
                                            fontSize: 14,
                                            color: replyCount > 10 ? '#c77' : '#666',
                                            fontWeight: replyCount > 10 ? 600 : 400,
                                        }}>
                                            {replyCount}
                                        </span>
                                        <span style={{
                                            width: 60,
                                            textAlign: 'center',
                                            fontSize: 14,
                                            color: '#666',
                                        }}>
                                            {formatRelativeTime(lastActivityTime)}
                                        </span>
                                    </div>
                                </div>
                            );
                        })
                    )}
                </div>
            </div>

            <Pagination
                currentPage={currentPage}
                totalPages={totalPages}
                onPageChange={handlePageChange}
            />

            <NewPostModal
                isOpen={showNewPostModal}
                onClose={() => setShowNewPostModal(false)}
                onRefresh={handleRefresh}
                categoryName={categoryName}
            />
        </main>
    );
}

export default ForumHome;
