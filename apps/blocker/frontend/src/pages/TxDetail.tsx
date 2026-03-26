import { useState, useEffect } from 'react';
import { useParams, Link } from 'react-router-dom';
import { get_tx_detail } from '../../wasm/pkg';
import type { TxDetail as TxDetailType } from '../types';
import { formatTimestamp, txTypeColor } from '../utils/format';
import HashLink from '../components/shared/HashLink';

function TxDetail() {
    const { hash } = useParams<{ hash: string }>();
    const [tx, setTx] = useState<TxDetailType | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        if (!hash) return;
        get_tx_detail(hash).then((t) => {
            setTx(t ?? null);
            setLoading(false);
        });
    }, [hash]);

    if (loading) return <div className="text-blocker-text-muted text-center py-12">Loading...</div>;
    if (!tx) return <div className="text-blocker-text-muted text-center py-12">Transaction not found</div>;

    return (
        <div>
            <h1 className="text-lg font-semibold text-blocker-text-primary mb-4">Transaction Details</h1>

            <div className="bg-blocker-surface border border-blocker-border rounded-lg">
                <div className="grid grid-cols-1 sm:grid-cols-[160px_1fr] text-sm">
                    <Row label="Tx Hash" value={tx.tx_hash} mono />
                    <Row label="Block">
                        <Link to={`/block/${tx.block_height}`} className="text-blocker-accent hover:text-blocker-accent-hover">
                            #{Number(tx.block_height)}
                        </Link>
                    </Row>
                    <Row label="Tx Index" value={String(tx.tx_index)} />
                    <Row label="Timestamp" value={formatTimestamp(Number(tx.timestamp))} />
                    <Row label="Type">
                        <span className={txTypeColor(tx.tx_type)}>{tx.tx_type}</span>
                    </Row>
                    {tx.function_sig && (
                        <Row label="Function signature" value={tx.function_sig} mono />
                    )}
                    {tx.sender && (
                        <Row label="Sender">
                            <HashLink hash={tx.sender} to={`/account/${tx.sender}`} />
                        </Row>
                    )}
                    {tx.target_site && (
                        <Row label="Target Site">
                            <HashLink hash={tx.target_site} to={`/site/${tx.target_site}`} />
                        </Row>
                    )}
                    <Row label="Nonce" value={tx.nonce} mono />
                    <Row label="Recent Block Height" value={String(tx.recent_block_height)} />
                </div>
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

export default TxDetail;
