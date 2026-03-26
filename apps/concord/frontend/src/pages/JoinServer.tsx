import React, { useEffect, useState } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { join_server } from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';

function JoinServer(): React.JSX.Element {
    const { serverId, serverKey } = useParams<{ serverId: string; serverKey: string }>();
    const navigate = useNavigate();
    const [error, setError] = useState('');

    useEffect(() => {
        if (!serverId || !serverKey) return;
        const id = Number(serverId);
        if (isNaN(id)) {
            setError('Invalid server ID');
            return;
        }
        join_server(BigInt(id), serverKey)
            .then(txHash => txHash ? await_tx_inclusion(txHash) : Promise.resolve())
            .then(() => navigate(`/server/${id}`, { replace: true }))
            .catch(() => setError('Failed to join server'));
    }, [serverId, serverKey, navigate]);

    if (error) {
        return (
            <div className="flex-1 flex items-center justify-center">
                <div className="text-center">
                    <p className="text-dc-text-muted mb-4">{error}</p>
                    <button onClick={() => navigate('/dms')} className="text-dc-blurple hover:underline text-sm">
                        Back to Messages
                    </button>
                </div>
            </div>
        );
    }

    return (
        <div className="flex-1 flex items-center justify-center">
            <p className="text-dc-text-muted">Joining server...</p>
        </div>
    );
}

export default JoinServer;
