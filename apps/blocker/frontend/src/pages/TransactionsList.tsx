import { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { get_tx_count, get_txs_page, get_tx_detail } from '../../wasm/pkg';
import type { TxDetail } from '../types';
import { txTypeColor, formatRelativeTime, truncateHash } from '../utils/format';
import HashLink from '../components/shared/HashLink';
import Pagination from '../components/shared/Pagination';
import { PAGE_SIZE } from '../config';

function TransactionsList() {
    const [txs, setTxs] = useState<TxDetail[]>([]);
    const [total, setTotal] = useState(0);
    const [page, setPage] = useState(0);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        loadPage(0);
    }, []);

    const loadPage = async (p: number) => {
        setLoading(true);
        setPage(p);

        const count = Number(await get_tx_count());
        setTotal(count);

        if (count > 0) {
            const totalPages = Math.ceil(count / PAGE_SIZE);
            const reversePage = Math.max(0, totalPages - 1 - p);
            const hashes = await get_txs_page(BigInt(reversePage));
            const reversed = [...hashes].reverse();
            const results = await Promise.all(reversed.map(h => get_tx_detail(h)));
            const details = results.filter(Boolean) as TxDetail[];
            setTxs(details);
        } else {
            setTxs([]);
        }
        setLoading(false);
    };

    if (loading && txs.length === 0) {
        return <div className="text-blocker-text-muted text-center py-12">Loading...</div>;
    }

    return (
        <div>
            <h1 className="text-lg font-semibold text-blocker-text-primary mb-4">Transactions ({total.toLocaleString()})</h1>

            <div className="bg-blocker-surface border border-blocker-border rounded-lg overflow-x-auto">
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
                        {txs.map((tx) => (
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
                {txs.length === 0 && (
                    <div className="text-center py-6 text-blocker-text-muted">No transactions yet</div>
                )}
                <Pagination currentPage={page} totalItems={total} pageSize={PAGE_SIZE} onPageChange={loadPage} />
            </div>
        </div>
    );
}

export default TransactionsList;
