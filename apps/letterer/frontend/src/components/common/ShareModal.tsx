import React, { useEffect, useState } from 'react';
import { X } from 'lucide-react';
import { get_document_key_hex } from '../../../wasm/pkg';

interface ShareModalProps {
    isOpen: boolean;
    onClose: () => void;
    docId: string;
}

function ShareModal({ isOpen, onClose, docId }: ShareModalProps): React.JSX.Element | null {
    const [docKeyHex, setDocKeyHex] = useState<string | null>(null);
    const [copied, setCopied] = useState(false);

    useEffect(() => {
        if (!isOpen) return;
        get_document_key_hex(docId).then(key => setDocKeyHex(key ?? null));
    }, [isOpen, docId]);

    if (!isOpen) return null;

    const inviteLink = docKeyHex ? `https://letterer.vastrum.net/share/${docKeyHex}` : '';

    const handleCopy = () => {
        navigator.clipboard.writeText(inviteLink);
        setCopied(true);
        setTimeout(() => setCopied(false), 2000);
    };

    return (
        <div style={{
            position: 'fixed',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backgroundColor: 'rgba(0,0,0,0.4)',
            zIndex: 1000,
        }} onClick={onClose}>
            <div style={{
                backgroundColor: '#fff',
                borderRadius: 12,
                width: 480,
                maxHeight: '80vh',
                overflow: 'auto',
                boxShadow: '0 20px 60px rgba(0,0,0,0.15)',
            }} onClick={e => e.stopPropagation()}>
                <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'space-between',
                    padding: '16px 20px',
                    borderBottom: '1px solid #e5e7eb',
                }}>
                    <h2 style={{ margin: 0, fontSize: 16, fontWeight: 600 }}>Share Document</h2>
                    <button onClick={onClose} style={{ background: 'none', border: 'none', color: '#6b7280' }}>
                        <X size={20} />
                    </button>
                </div>

                <div style={{ padding: 20 }}>
                    <p style={{ fontSize: 13, color: '#6b7280', marginTop: 0, marginBottom: 12 }}>
                        Share this invite link. Anyone with the link can view and edit this document.
                    </p>
                    {inviteLink && (
                        <>
                            <div style={{
                                padding: '8px 12px',
                                backgroundColor: '#f3f4f6',
                                borderRadius: 6,
                                fontSize: 13,
                                fontFamily: 'monospace',
                                wordBreak: 'break-all',
                                userSelect: 'all',
                            }}>
                                {inviteLink}
                            </div>
                            <button
                                onClick={handleCopy}
                                style={{
                                    marginTop: 12,
                                    width: '100%',
                                    padding: '8px 0',
                                    borderRadius: 6,
                                    border: 'none',
                                    fontSize: 13,
                                    fontWeight: 600,
                                    cursor: 'pointer',
                                    backgroundColor: copied ? '#16a34a' : '#2563eb',
                                    color: '#fff',
                                }}
                            >
                                {copied ? 'Copied!' : 'Copy Link'}
                            </button>
                        </>
                    )}
                </div>
            </div>
        </div>
    );
}

export default ShareModal;
