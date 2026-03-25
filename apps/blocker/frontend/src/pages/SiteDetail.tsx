import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { get_site_detail, get_site_txs, get_tx_detail } from '../../wasm/pkg';
import type { SiteDetail as SiteDetailType, TxDetail as TxDetailType } from '../types';
import { txTypeColor, formatRelativeTime, truncateHash } from '../utils/format';
import HashLink from '../components/shared/HashLink';
import Pagination from '../components/shared/Pagination';
import { PAGE_SIZE, getSiteUrl } from '../config';

function SiteDetail() {
    const { id } = useParams<{ id: string }>();
    const [site, setSite] = useState<SiteDetailType | null>(null);
    const [txs, setTxs] = useState<TxDetailType[]>([]);
    const [txPage, setTxPage] = useState(0);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        if (!id) return;
        get_site_detail(id).then((s) => {
            setSite(s ?? null);
            setLoading(false);
        });
    }, [id]);

    useEffect(() => {
        if (!id || !site) return;
        loadTxPage(0);
    }, [id, site]);

    const loadTxPage = async (p: number) => {
        if (!id || !site) return;
        setTxPage(p);
        const totalPages = Math.ceil(site.tx_count / PAGE_SIZE);
        const reversePage = Math.max(0, totalPages - 1 - p);
        const hashes = await get_site_txs(id, BigInt(reversePage));
        const reversed = [...hashes].reverse();
        const results = await Promise.all(reversed.map(h => get_tx_detail(h)));
        const details = results.filter(Boolean) as TxDetailType[];
        setTxs(details);
    };

    if (loading) return <div className="text-blocker-text-muted text-center py-12">Loading...</div>;
    if (!site) return <div className="text-blocker-text-muted text-center py-12">Site not found</div>;

    return (
        <div>
            <h1 className="text-lg font-semibold text-blocker-text-primary mb-4">Site Details</h1>

            <div className="bg-blocker-surface border border-blocker-border rounded-lg mb-6">
                <div className="grid grid-cols-1 sm:grid-cols-[160px_1fr] text-sm">
                    <Row label="Site ID" value={site.site_id} mono />
                    {site.module_id && <Row label="Module ID" value={site.module_id} mono />}
                    <Row label="Deploy Tx">
                        <HashLink hash={site.deploy_tx} to={`/tx/${site.deploy_tx}`} />
                    </Row>
                    <Row label="Deploy Block">
                        <Link to={`/block/${site.block_height}`} className="text-blocker-accent hover:text-blocker-accent-hover">
                            #{Number(site.block_height)}
                        </Link>
                    </Row>
                    <Row label="Domain">
                        {site.domain ? (
                            <span className="text-blocker-success">{site.domain}</span>
                        ) : (
                            <span className="text-blocker-text-muted">None</span>
                        )}
                    </Row>
                    <Row label="Total Calls" value={String(site.tx_count)} />
                    <Row label="Visit">
                        <a
                            href={getSiteUrl(site.domain ?? site.site_id)}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-blocker-accent hover:text-blocker-accent-hover transition-colors"
                        >
                            {site.domain ?? truncateHash(site.site_id, 8)}
                        </a>
                    </Row>
                </div>
            </div>

            {/* Transaction list */}
            <div className="bg-blocker-surface border border-blocker-border rounded-lg overflow-x-auto">
                <div className="px-4 py-3 border-b border-blocker-border">
                    <h2 className="text-sm font-medium text-blocker-text-primary">Transactions ({site.tx_count})</h2>
                </div>
                {txs.length > 0 ? (
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
                                <tr key={tx.tx_hash} className="border-b border-blocker-border last:border-0 hover:bg-blocker-surface-hover">
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
                ) : (
                    <div className="text-center py-6 text-blocker-text-muted">No transactions to this site</div>
                )}
                <Pagination currentPage={txPage} totalItems={site.tx_count} pageSize={PAGE_SIZE} onPageChange={loadTxPage} />
            </div>
        </div>
    );
}

function Row({ label, value, mono, children }: { label: string; value?: string; mono?: boolean; children?: React.ReactNode }) {
    return (
        <>
            <div className="px-4 py-2.5 text-blocker-text-muted border-b border-blocker-border">{label}</div>
            <div className={`px-4 py-2.5 text-blocker-text-primary border-b border-blocker-border break-all ${mono ? 'font-mono text-xs' : ''}`}>
                {children ?? value}
            </div>
        </>
    );
}

export default SiteDetail;
