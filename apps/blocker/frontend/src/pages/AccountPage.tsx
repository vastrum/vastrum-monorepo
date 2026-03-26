import { useState, useEffect } from 'react';
import { useParams } from 'react-router-dom';
import { get_account_tx_count, get_account_txs, get_tx_detail } from '../../wasm/pkg';
import type { TxDetail } from '../types';
import { formatRelativeTime, txTypeColor, truncateHash } from '../utils/format';
import HashLink from '../components/shared/HashLink';
import Pagination from '../components/shared/Pagination';
import { PAGE_SIZE } from '../config';

function AccountPage() {
    const { pubkey } = useParams<{ pubkey: string }>();
    const [txCount, setTxCount] = useState(0);
    const [txDetails, setTxDetails] = useState<TxDetail[]>([]);
    const [page, setPage] = useState(0);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        if (!pubkey) return;
        loadPage(0);
    }, [pubkey]);

    const loadPage = async (p: number) => {
        if (!pubkey) return;
        setLoading(true);
        setPage(p);

        const count = Number(await get_account_tx_count(pubkey));
        setTxCount(count);

        // Calculate reverse page (latest txs first)
        const totalPages = Math.ceil(count / PAGE_SIZE);
        const reversePage = Math.max(0, totalPages - 1 - p);

        const hashes = await get_account_txs(pubkey, BigInt(reversePage));
        const reversed = [...hashes].reverse();
        const results = await Promise.all(reversed.map(h => get_tx_detail(h)));
        const details = results.filter(Boolean) as TxDetail[];
        setTxDetails(details);
        setLoading(false);
    };

    if (loading && txDetails.length === 0) {
        return <div className="text-blocker-text-muted text-center py-12">Loading...</div>;
    }

    return (
        <div>
            <h1 className="text-lg font-semibold text-blocker-text-primary mb-4">Account</h1>

            <div className="bg-blocker-surface border border-blocker-border rounded-lg mb-6">
                <div className="grid grid-cols-1 sm:grid-cols-[160px_1fr] text-sm">
                    <div className="px-4 py-2.5 text-blocker-text-muted border-b border-blocker-border">Public Key</div>
                    <div className="px-4 py-2.5 text-blocker-text-primary border-b border-blocker-border font-mono text-xs break-all">{pubkey}</div>
                    <div className="px-4 py-2.5 text-blocker-text-muted border-b border-blocker-border">Transactions</div>
                    <div className="px-4 py-2.5 text-blocker-text-primary border-b border-blocker-border">{txCount}</div>
                </div>
            </div>

            <div className="bg-blocker-surface border border-blocker-border rounded-lg overflow-x-auto">
                <div className="px-4 py-3 border-b border-blocker-border">
                    <h2 className="text-sm font-medium text-blocker-text-primary">Transaction History</h2>
                </div>
                {txDetails.length > 0 ? (
                    <table className="w-full text-sm">
                        <thead>
                            <tr className="text-blocker-text-muted text-xs uppercase tracking-wider border-b border-blocker-border">
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium">Tx Hash</th>
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium hidden sm:table-cell">Block</th>
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium">Type</th>
                                <th className="text-left px-2 sm:px-4 py-2.5 font-medium">Time</th>
                            </tr>
                        </thead>
                        <tbody>
                            {txDetails.map((tx) => (
                                <tr key={tx.tx_hash} className="border-b border-blocker-border last:border-0 hover:bg-blocker-surface-hover">
                                    <td className="px-2 sm:px-4 py-2.5">
                                        <HashLink hash={tx.tx_hash} to={`/tx/${tx.tx_hash}`} />
                                    </td>
                                    <td className="px-2 sm:px-4 py-2.5 hidden sm:table-cell">
                                        <HashLink hash={String(Number(tx.block_height))} to={`/block/${tx.block_height}`} truncate={20} />
                                    </td>
                                    <td className={`px-2 sm:px-4 py-2.5 ${txTypeColor(tx.tx_type)}`}>
                                        {tx.tx_type}
                                        {tx.function_sig && (
                                            <span className="ml-1.5 font-mono text-blocker-text-muted text-xs">
                                                {truncateHash(tx.function_sig, 6)}
                                            </span>
                                        )}
                                    </td>
                                    <td className="px-2 sm:px-4 py-2.5 text-blocker-text-muted">{formatRelativeTime(Number(tx.timestamp))}</td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                ) : (
                    <div className="text-center py-6 text-blocker-text-muted">No transactions found</div>
                )}
                <Pagination currentPage={page} totalItems={txCount} pageSize={PAGE_SIZE} onPageChange={loadPage} />
            </div>
        </div>
    );
}

export default AccountPage;
