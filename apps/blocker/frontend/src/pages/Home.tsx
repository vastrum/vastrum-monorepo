import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { get_latest_height, get_block, get_site_count, get_tx_count, get_txs_page, get_tx_detail } from '../../wasm/pkg';
import type { BlockSummary, TxDetail } from '../types';
import { formatRelativeTime, txTypeColor, truncateHash } from '../utils/format';
import HashLink from '../components/shared/HashLink';
import { PAGE_SIZE } from '../config';

function Home() {
    const [blocks, setBlocks] = useState<BlockSummary[]>([]);
    const [recentTxs, setRecentTxs] = useState<TxDetail[]>([]);
    const [latestHeight, setLatestHeight] = useState(0);
    const [siteCount, setSiteCount] = useState(0);

    const [txCount, setTxCount] = useState(0);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        fetchData();
    }, []);

    const fetchData = async () => {
        const height = Number(await get_latest_height());
        setLatestHeight(height);

        const start = Math.max(1, height - 4);
        const heights = [];
        for (let h = height; h >= start; h--) heights.push(h);
        const blockResults = await Promise.all(heights.map(h => get_block(BigInt(h))));
        const blockList = blockResults.filter(Boolean) as BlockSummary[];
        setBlocks(blockList);

        const totalTxs = Number(await get_tx_count());
        setTxCount(totalTxs);

        // Fetch latest page of tx hashes, then resolve details
        if (totalTxs > 0) {
            const lastPage = Math.floor((totalTxs - 1) / PAGE_SIZE);
            const hashes = await get_txs_page(BigInt(lastPage));
            const top5 = [...hashes].reverse().slice(0, 5);
            const results = await Promise.all(top5.map(h => get_tx_detail(h)));
            const details = results.filter(Boolean) as TxDetail[];
            setRecentTxs(details);
        }

        setSiteCount(Number(await get_site_count()));
        setLoading(false);
    };

    if (loading) {
        return <div className="text-blocker-text-muted text-center py-12">Loading...</div>;
    }

    return (
        <div>
            {/* Stats */}
            <div className="grid grid-cols-2 sm:grid-cols-3 gap-3 sm:gap-4 mb-6">
                <div className="bg-blocker-surface border border-blocker-border rounded-lg p-4">
                    <div className="text-blocker-text-muted text-xs uppercase tracking-wider mb-1">Block Height</div>
                    <div className="text-2xl font-semibold text-blocker-text-primary">{latestHeight.toLocaleString()}</div>
                </div>
                <div className="bg-blocker-surface border border-blocker-border rounded-lg p-4">
                    <div className="text-blocker-text-muted text-xs uppercase tracking-wider mb-1">Transactions</div>
                    <div className="text-2xl font-semibold text-blocker-text-primary">{txCount.toLocaleString()}</div>
                </div>
                <div className="bg-blocker-surface border border-blocker-border rounded-lg p-4">
                    <div className="text-blocker-text-muted text-xs uppercase tracking-wider mb-1">Sites Deployed</div>
                    <div className="text-2xl font-semibold text-blocker-text-primary">{siteCount.toLocaleString()}</div>
                </div>
            </div>

            <div className="space-y-4">
                {/* Recent Blocks Table */}
                <div className="bg-blocker-surface border border-blocker-border rounded-lg overflow-x-auto">
                    <div className="px-4 py-3 border-b border-blocker-border flex items-center justify-between">
                        <h2 className="text-sm font-medium text-blocker-text-primary">Latest Blocks</h2>
                        <Link to="/blocks" className="text-xs text-blocker-accent hover:text-blocker-accent-hover transition-colors">View all</Link>
                    </div>
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="text-blocker-text-muted text-xs uppercase tracking-wider border-b border-blocker-border">
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium">Height</th>
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium hidden sm:table-cell">Hash</th>
                                <th className="text-right px-2 sm:px-4 py-2.5 font-medium">Txns</th>
                                <th className="text-right px-2 sm:px-4 py-2.5 font-medium">Age</th>
                            </tr>
                        </thead>
                        <tbody>
                            {blocks.map((block) => (
                                <tr key={Number(block.height)} className="border-b border-blocker-border last:border-0 hover:bg-blocker-surface-hover transition-colors">
                                    <td className="px-2 sm:px-4 py-2.5">
                                        <Link to={`/block/${block.height}`} className="text-blocker-accent hover:text-blocker-accent-hover">
                                            {Number(block.height)}
                                        </Link>
                                    </td>
                                    <td className="px-2 sm:px-4 py-2.5 hidden sm:table-cell">
                                        <HashLink hash={block.block_hash} to={`/block/${block.height}`} truncate={6} />
                                    </td>
                                    <td className="px-2 sm:px-4 py-2.5 text-right text-blocker-text-secondary">{Number(block.tx_count)}</td>
                                    <td className="px-2 sm:px-4 py-2.5 text-right text-blocker-text-muted">{formatRelativeTime(Number(block.timestamp))}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                    {blocks.length === 0 && (
                        <div className="text-center py-8 text-blocker-text-muted">No blocks yet</div>
                    )}
                </div>

                {/* Latest Transactions Table */}
                <div className="bg-blocker-surface border border-blocker-border rounded-lg overflow-x-auto">
                    <div className="px-4 py-3 border-b border-blocker-border flex items-center justify-between">
                        <h2 className="text-sm font-medium text-blocker-text-primary">Latest Transactions</h2>
                        <Link to="/transactions" className="text-xs text-blocker-accent hover:text-blocker-accent-hover transition-colors">View all</Link>
                    </div>
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="text-blocker-text-muted text-xs uppercase tracking-wider border-b border-blocker-border">
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium">Tx Hash</th>
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium">Type</th>
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium hidden sm:table-cell">Sender</th>
                                <th className="text-right px-2 sm:px-4 py-2.5 font-medium hidden sm:table-cell">Block</th>
                                <th className="text-right px-2 sm:px-4 py-2.5 font-medium">Age</th>
                            </tr>
                        </thead>
                        <tbody>
                            {recentTxs.map((tx) => (
                                <tr key={tx.tx_hash} className="border-b border-blocker-border last:border-0 hover:bg-blocker-surface-hover transition-colors">
                                    <td className="px-2 sm:px-4 py-2.5">
                                        <HashLink hash={tx.tx_hash} to={`/tx/${tx.tx_hash}`} truncate={6} />
                                    </td>
                                    <td className={`px-2 sm:px-4 py-2.5 ${txTypeColor(tx.tx_type)}`}>
                                        {tx.tx_type}
                                        {tx.function_sig && (
                                            <span className="ml-1.5 font-mono text-blocker-text-muted text-xs">
                                                {truncateHash(tx.function_sig, 6)}
                                            </span>
                                        )}
                                    </td>
                                    <td className="px-2 sm:px-4 py-2.5 hidden sm:table-cell">
                                        {tx.sender ? <HashLink hash={tx.sender} to={`/account/${tx.sender}`} truncate={6} /> : <span className="text-blocker-text-muted">-</span>}
                                    </td>
                                    <td className="px-2 sm:px-4 py-2.5 text-right hidden sm:table-cell">
                                        <Link to={`/block/${tx.block_height}`} className="text-blocker-accent hover:text-blocker-accent-hover">
                                            {Number(tx.block_height)}
                                        </Link>
                                    </td>
                                    <td className="px-2 sm:px-4 py-2.5 text-right text-blocker-text-muted">{formatRelativeTime(Number(tx.timestamp))}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                    {recentTxs.length === 0 && (
                        <div className="text-center py-8 text-blocker-text-muted">No transactions yet</div>
                    )}
                </div>
            </div>
        </div>
    );
}

export default Home;
