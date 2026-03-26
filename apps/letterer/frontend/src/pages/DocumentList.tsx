import React, { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Plus, FileText, Trash2, Clock } from 'lucide-react';
import { get_my_documents, create_document, delete_document } from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import type { JSDocumentMeta } from '../../wasm/pkg';
import { formatRelativeTime } from '@/utils/timeUtils';

function DocumentList(): React.JSX.Element {
    const [documents, setDocuments] = useState<JSDocumentMeta[]>([]);
    const [loading, setLoading] = useState(true);
    const navigate = useNavigate();

    const fetchDocs = useCallback(async () => {
        const docs = await get_my_documents();
        docs.sort((a, b) => Number(b.last_modified) - Number(a.last_modified));
        setDocuments(docs);
        setLoading(false);
    }, []);

    useEffect(() => {
        fetchDocs();
        const interval = setInterval(fetchDocs, 10_000);
        return () => clearInterval(interval);
    }, [fetchDocs]);

    const handleCreate = async () => {
        const txHash = await create_document('Untitled');
        if (txHash) {
            await await_tx_inclusion(txHash);
            const docs = await get_my_documents();
            docs.sort((a, b) => Number(b.last_modified) - Number(a.last_modified));
            setDocuments(docs);
            if (docs.length > 0) {
                navigate(`/doc/${docs[0].id}`);
            }
        }
    };

    const handleDelete = async (e: React.MouseEvent, docId: string) => {
        e.stopPropagation();
        const txHash = await delete_document(docId);
        if (txHash) {
            await await_tx_inclusion(txHash);
            fetchDocs();
        }
    };

    if (loading) {
        return (
            <div style={{ display: 'flex', justifyContent: 'center', padding: 60 }}>
                <p style={{ color: '#9ca3af' }}>Loading documents...</p>
            </div>
        );
    }

    return (
        <div style={{ maxWidth: 800, margin: '0 auto', padding: '32px 24px' }}>
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 24 }}>
                <h1 style={{ margin: 0, fontSize: 22, fontWeight: 600, color: '#111' }}>My Documents</h1>
                <button
                    onClick={handleCreate}
                    style={{
                        display: 'flex',
                        alignItems: 'center',
                        gap: 6,
                        padding: '8px 16px',
                        backgroundColor: '#2563eb',
                        color: '#fff',
                        border: 'none',
                        borderRadius: 8,
                        fontSize: 14,
                        fontWeight: 500,
                    }}
                    className="header-btn"
                >
                    <Plus size={18} />
                    New Document
                </button>
            </div>

            {documents.length === 0 ? (
                <div style={{
                    textAlign: 'center',
                    padding: '60px 20px',
                    color: '#9ca3af',
                }}>
                    <FileText size={48} style={{ marginBottom: 12, opacity: 0.5, display: 'block', margin: '0 auto 12px' }} />
                    <p style={{ fontSize: 15 }}>No documents created yet</p>
                </div>
            ) : (
                <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                    {documents.map(doc => (
                        <div
                            key={doc.id}
                            onClick={() => navigate(`/doc/${doc.id}`)}
                            className="doc-row"
                            style={{
                                display: 'flex',
                                alignItems: 'center',
                                gap: 12,
                                padding: '14px 16px',
                                border: '1px solid #e5e7eb',
                                borderRadius: 8,
                                cursor: 'pointer',
                            }}
                        >
                            <FileText size={20} color="#6b7280" />
                            <div style={{ flex: 1, minWidth: 0 }}>
                                <span style={{
                                    fontSize: 15,
                                    fontWeight: 500,
                                    color: '#111',
                                    overflow: 'hidden',
                                    textOverflow: 'ellipsis',
                                    whiteSpace: 'nowrap',
                                    display: 'block',
                                }}>
                                    {doc.title || 'Untitled'}
                                </span>
                                <div style={{
                                    display: 'flex',
                                    alignItems: 'center',
                                    gap: 4,
                                    marginTop: 4,
                                    fontSize: 12,
                                    color: '#9ca3af',
                                }}>
                                    <Clock size={12} />
                                    {formatRelativeTime(Number(doc.last_modified))}
                                </div>
                            </div>
                            {doc.created_by_me && (
                                <button
                                    onClick={(e) => handleDelete(e, doc.id)}
                                    title="Delete document"
                                    style={{
                                        display: 'flex',
                                        alignItems: 'center',
                                        justifyContent: 'center',
                                        width: 32,
                                        height: 32,
                                        border: 'none',
                                        borderRadius: 4,
                                        background: 'none',
                                        color: '#d1d5db',
                                    }}
                                    className="delete-btn"
                                >
                                    <Trash2 size={16} />
                                </button>
                            )}
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}

export default DocumentList;
