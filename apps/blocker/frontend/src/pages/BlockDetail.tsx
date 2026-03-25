import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { get_block, get_block_txs } from '../../wasm/pkg';
import type { BlockSummary, TxSummary } from '../types';
import { formatTimestamp, txTypeColor, truncateHash } from '../utils/format';
import HashLink from '../components/shared/HashLink';

function BlockDetail() {
    const { height } = useParams<{ height: string }>();
    const [block, setBlock] = useState<BlockSummary | null>(null);
    const [txs, setTxs] = useState<TxSummary[]>([]);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        if (!height) return;
        const h = BigInt(height);
        Promise.all([get_block(h), get_block_txs(h)]).then(([b, t]) => {
            setBlock(b ?? null);
            setTxs(t);
            setLoading(false);
        });
    }, [height]);

    if (loading) return <div className="text-blocker-text-muted text-center py-12">Loading...</div>;
    if (!block) return <div className="text-blocker-text-muted text-center py-12">Block not found</div>;

    const h = Number(block.height);

    return (
        <div>
            <div className="flex items-center gap-2 mb-4">
                <h1 className="text-lg font-semibold text-blocker-text-primary">Block #{h}</h1>
                <div className="flex gap-1 ml-auto">
                    {h > 1 && (
                        <Link to={`/block/${h - 1}`} className="px-2 py-1 text-xs border border-blocker-border rounded text-blocker-text-secondary hover:bg-blocker-surface-hover">
                            Prev
                        </Link>
                    )}
                    <Link to={`/block/${h + 1}`} className="px-2 py-1 text-xs border border-blocker-border rounded text-blocker-text-secondary hover:bg-blocker-surface-hover">
                        Next
                    </Link>
                </div>
            </div>

            {/* Block Info */}
            <div className="bg-blocker-surface border border-blocker-border rounded-lg mb-6">
                <div className="grid grid-cols-1 sm:grid-cols-[160px_1fr] text-sm">
                    <Row label="Block Hash" value={block.block_hash} mono />
                    <Row label="Previous Hash" value={block.previous_block_hash} mono />
                    <Row label="Timestamp" value={formatTimestamp(Number(block.timestamp))} />
                    <Row label="Transactions" value={String(Number(block.tx_count))} />
                </div>
            </div>

            {/* Transactions */}
            <div className="bg-blocker-surface border border-blocker-border rounded-lg overflow-x-auto">
                <div className="px-4 py-3 border-b border-blocker-border">
                    <h2 className="text-sm font-medium text-blocker-text-primary">Transactions ({txs.length})</h2>
                </div>
                {txs.length > 0 ? (
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="text-blocker-text-muted text-xs uppercase tracking-wider border-b border-blocker-border">
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium">Tx Hash</th>
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium">Type</th>
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium hidden sm:table-cell">Sender</th>
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium hidden sm:table-cell">Target</th>
                            </tr>
                        </thead>
                        <tbody>
                            {txs.map((tx) => (
                                <tr key={tx.tx_hash} className="border-b border-blocker-border last:border-0 hover:bg-blocker-surface-hover">
                                    <td className="px-2 sm:px-4 py-2.5">
                                        <HashLink hash={tx.tx_hash} to={`/tx/${tx.tx_hash}`} />
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
                                    <td className="px-2 sm:px-4 py-2.5 hidden sm:table-cell">
                                        {tx.target_site ? (
                                            <HashLink hash={tx.target_site} to={`/site/${tx.target_site}`} truncate={6} />
                                        ) : (
                                            <span className="text-blocker-text-muted">-</span>
                                        )}
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                ) : (
                    <div className="text-center py-6 text-blocker-text-muted">No transactions in this block</div>
                )}
            </div>
        </div>
    );
}

function Row({ label, value, mono }: { label: string; value: string; mono?: boolean }) {
    return (
        <>
            <div className="px-4 py-2.5 text-blocker-text-muted border-b border-blocker-border">{label}</div>
            <div className={`px-4 py-2.5 text-blocker-text-primary border-b border-blocker-border break-all ${mono ? 'font-mono text-xs' : ''}`}>
                {value}
            </div>
        </>
    );
}

export default BlockDetail;
