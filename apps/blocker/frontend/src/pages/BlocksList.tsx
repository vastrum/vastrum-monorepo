import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { get_latest_height, get_block } from '../../wasm/pkg';
import type { BlockSummary } from '../types';
import { formatRelativeTime } from '../utils/format';
import HashLink from '../components/shared/HashLink';
import Pagination from '../components/shared/Pagination';
import { PAGE_SIZE } from '../config';

function BlocksList() {
    const [blocks, setBlocks] = useState<BlockSummary[]>([]);
    const [latestHeight, setLatestHeight] = useState(0);
    const [page, setPage] = useState(0);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        loadPage(0);
    }, []);

    const loadPage = async (p: number) => {
        setLoading(true);
        setPage(p);
        const height = Number(await get_latest_height());
        setLatestHeight(height);

        const startHeight = height - p * PAGE_SIZE;
        const endHeight = Math.max(1, startHeight - (PAGE_SIZE - 1));
        const heights = [];
        for (let h = startHeight; h >= endHeight; h--) heights.push(h);
        const blockResults = await Promise.all(heights.map(h => get_block(BigInt(h))));
        const blockList = blockResults.filter(Boolean) as BlockSummary[];
        setBlocks(blockList);
        setLoading(false);
    };

    if (loading && blocks.length === 0) {
        return <div className="text-blocker-text-muted text-center py-12">Loading...</div>;
    }

    return (
        <div>
            <h1 className="text-lg font-semibold text-blocker-text-primary mb-4">Blocks ({latestHeight.toLocaleString()})</h1>

            <div className="bg-blocker-surface border border-blocker-border rounded-lg overflow-x-auto">
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
                    <div className="text-center py-6 text-blocker-text-muted">No blocks yet</div>
                )}
                <Pagination currentPage={page} totalItems={latestHeight} pageSize={PAGE_SIZE} onPageChange={loadPage} />
            </div>
        </div>
    );
}

export default BlocksList;
