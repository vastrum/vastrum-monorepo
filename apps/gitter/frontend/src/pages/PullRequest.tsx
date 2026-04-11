import React, { useState, useEffect } from 'react';
import { Link, useParams, useNavigate } from 'react-router-dom';
import {
    MessageSquare,
    GitCommit,
    FileText
} from 'lucide-react';
import {
    type PullRequest as PullRequestType,
    type GetPullRequestDetail,
    get_pull_request_detail,
    reply_to_pull_request,
    merge_pull_request
} from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import { truncateAddress } from '../utils/avatarGenerator';
import ConversationTab from '../components/pull-request/tabs/ConversationTab';
import CommitsTab from '../components/pull-request/tabs/CommitsTab';
import FilesChangedTab from '../components/pull-request/tabs/FilesChangedTab';

type PRTab = 'conversation' | 'commits' | 'files';

function PullRequest(): React.JSX.Element {
    const { repoId, id } = useParams<{ repoId: string; id: string }>();
    const navigate = useNavigate();
    const [activeTab, setActiveTab] = useState<PRTab>('conversation');
    const [prDetail, setPrDetail] = useState<GetPullRequestDetail | null>(null);
    const [loading, setLoading] = useState(true);

    const fetchData = async (): Promise<void> => {
        if (!repoId || !id) return;
        try {
            const detail = await get_pull_request_detail(repoId, BigInt(id));
            setPrDetail(detail);
        } catch (error) {
            console.error('Failed to fetch PR data:', error);
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        fetchData();
    }, [repoId, id]);

    const pr: PullRequestType | null = prDetail?.pull_request ?? null;

    const handleComment = async (content: string): Promise<void> => {
        if (!pr || !repoId) return;
        const txHash = await reply_to_pull_request(content, repoId, BigInt(pr.id));
        await await_tx_inclusion(txHash);
        fetchData();
    };

    const handleMerge = async (): Promise<void> => {
        if (!pr || !repoId) return;
        const txHash = await merge_pull_request(repoId, BigInt(pr.id));
        await await_tx_inclusion(txHash);
        fetchData();
    };

    if (loading) {
        return <div className="max-w-7xl mx-auto px-5 py-5">Loading...</div>;
    }

    if (!pr || !repoId) {
        return (
            <div className="max-w-7xl mx-auto px-5 py-5">
                <p>Pull request not found.</p>
                <button onClick={() => navigate(`/repo/${repoId}`)} className="btn-primary mt-4">Back to repository</button>
            </div>
        );
    }

    const getStatusBadge = () => {
        if (!pr.is_open) {
            return (
                <span className="inline-flex items-center gap-2 bg-app-accent-purple text-white px-3 py-1.5 rounded-full text-sm font-semibold">
                    <svg viewBox="0 0 16 16" className="w-4 h-4 fill-current">
                        <path d="M5.45 5.154A4.25 4.25 0 0 0 9.25 7.5h1.378a2.251 2.251 0 1 1 0 1.5H9.25A5.734 5.734 0 0 1 5 7.123v3.505a2.25 2.25 0 1 1-1.5 0V5.372a2.25 2.25 0 1 1 1.95-.218ZM4.25 13.5a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5Zm8.5-4.5a.75.75 0 1 0 0-1.5.75.75 0 0 0 0 1.5ZM5 3.25a.75.75 0 1 0 0 .005V3.25Z"></path>
                    </svg>
                    Merged
                </span>
            );
        }
        return (
            <span className="inline-flex items-center gap-2 bg-app-accent-green text-white px-3 py-1.5 rounded-full text-sm font-semibold">
                <svg viewBox="0 0 16 16" className="w-4 h-4 fill-current">
                    <path d="M1.5 3.25a2.25 2.25 0 1 1 3 2.122v5.256a2.251 2.251 0 1 1-1.5 0V5.372A2.25 2.25 0 0 1 1.5 3.25Zm5.677-.177L9.573.677A.25.25 0 0 1 10 .854V2.5h1A2.5 2.5 0 0 1 13.5 5v5.628a2.251 2.251 0 1 1-1.5 0V5a1 1 0 0 0-1-1h-1v1.646a.25.25 0 0 1-.427.177L7.177 3.427a.25.25 0 0 1 0-.354ZM3.75 2.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Zm0 9.5a.75.75 0 1 0 0 1.5.75.75 0 0 0 0-1.5Zm8.25.75a.75.75 0 1 0 1.5 0 .75.75 0 0 0-1.5 0Z"></path>
                </svg>
                Open
            </span>
        );
    };

    const commitCount = prDetail?.commits_to_merge.length || 0;
    const fileCount = prDetail?.file_changes.length || 0;

    return (
        <div key={pr.id} className="max-w-7xl mx-auto px-5 py-5 md:px-6 md:py-6">
            {/* Breadcrumb */}
            <div className="flex items-center gap-2 mb-3 md:mb-4 text-xs md:text-sm overflow-x-auto scrollbar-thin pb-1">
                <Link to={`/repo/${repoId}`} className="text-app-accent-blue hover:underline whitespace-nowrap flex-shrink-0">{repoId}</Link>
                <span className="text-app-text-secondary flex-shrink-0">/</span>
                <Link to={`/repo/${repoId}/pulls`} className="text-app-accent-blue hover:underline whitespace-nowrap flex-shrink-0">Pull requests</Link>
                <span className="text-app-text-secondary flex-shrink-0">/</span>
                <span className="text-app-text-secondary whitespace-nowrap flex-shrink-0">#{pr.id}</span>
            </div>

            {/* PR Header */}
            <div className="mb-4 md:mb-6">
                <div className="flex items-start gap-3 md:gap-4 mb-3 md:mb-4">
                    <h1 className="text-xl md:text-2xl lg:text-3xl font-semibold flex-1">
                        {pr.title} <span className="text-app-text-secondary">#{pr.id}</span>
                    </h1>
                </div>

                <div className="flex items-center gap-3 flex-wrap">
                    {getStatusBadge()}
                    <span className="text-app-text-secondary text-sm">
                        <span className="text-app-text-primary font-medium">{truncateAddress(pr.from)}</span>
                        {' '}wants to merge <strong className="text-app-text-primary">{commitCount} commits</strong> from <strong className="text-app-text-primary">{pr.head_repo}:{pr.head_branch}</strong> into <strong className="text-app-text-primary">{pr.base_repo}:{pr.base_branch}</strong>
                    </span>
                </div>
            </div>

            {/* Tabs */}
            <div className="border-b border-app-border mb-4 md:mb-6">
                <div className="flex gap-0 overflow-x-auto scrollbar-thin">
                    <button
                        onClick={() => setActiveTab('conversation')}
                        className={`tab-item whitespace-nowrap ${activeTab === 'conversation' ? 'active' : ''}`}
                    >
                        <MessageSquare className="w-4 h-4 inline mr-2" />
                        Conversation
                    </button>
                    <button
                        onClick={() => setActiveTab('commits')}
                        className={`tab-item whitespace-nowrap ${activeTab === 'commits' ? 'active' : ''}`}
                    >
                        <GitCommit className="w-4 h-4 inline mr-2" />
                        Commits
                        <span className="ml-2 bg-app-bg-tertiary px-2 py-0.5 rounded-full text-xs font-semibold">{commitCount}</span>
                    </button>
                    <button
                        onClick={() => setActiveTab('files')}
                        className={`tab-item whitespace-nowrap ${activeTab === 'files' ? 'active' : ''}`}
                    >
                        <FileText className="w-4 h-4 inline mr-2" />
                        Files changed
                        <span className="ml-2 bg-app-bg-tertiary px-2 py-0.5 rounded-full text-xs font-semibold">{fileCount}</span>
                    </button>
                </div>
            </div>

            {/* Main Content */}
            <div className="grid grid-cols-1 lg:grid-cols-3 gap-5 md:gap-6">
                <div className="lg:col-span-2">
                    {activeTab === 'conversation' && <ConversationTab pr={pr} prDetail={prDetail} repoId={repoId} onComment={handleComment} onMerge={handleMerge} />}
                    {activeTab === 'commits' && <CommitsTab commits={prDetail?.commits_to_merge || []} sourceBranch={`${pr.head_repo}:${pr.head_branch}`} />}
                    {activeTab === 'files' && <FilesChangedTab fileChanges={prDetail?.file_changes || []} />}
                </div>
            </div>
        </div>
    );
}

export default PullRequest;
