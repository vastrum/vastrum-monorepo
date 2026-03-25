import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { join_document } from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';

function JoinDocument(): React.JSX.Element {
    const { docKey } = useParams<{ docKey: string }>();
    const navigate = useNavigate();
    const [error, setError] = useState('');

    useEffect(() => {
        if (!docKey) return;
        join_document(docKey)
            .then(txHash => txHash ? await_tx_inclusion(txHash) : Promise.resolve())
            .then(() => navigate('/', { replace: true }))
            .catch(() => setError('Failed to join document'));
    }, [docKey, navigate]);

    if (error) {
        return (
            <div style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                padding: 60,
            }}>
                <div style={{ textAlign: 'center' }}>
                    <p style={{ color: '#ef4444', marginBottom: 16 }}>{error}</p>
                    <button
                        onClick={() => navigate('/')}
                        style={{
                            color: '#2563eb',
                            background: 'none',
                            border: 'none',
                            fontSize: 14,
                            cursor: 'pointer',
                        }}
                    >
                        Back to Documents
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            padding: 60,
        }}>
            <p style={{ color: '#9ca3af' }}>Joining document...</p>
        </div>
    );
}

export default JoinDocument;
