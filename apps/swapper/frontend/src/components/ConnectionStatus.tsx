import { useState, useEffect, useRef } from 'react';

interface ConnectionStatusProps {
    isConnected: boolean;
    isLoading: boolean;
    blockNumber: bigint | null;
}

export function ConnectionStatus({ isConnected, isLoading, blockNumber }: ConnectionStatusProps) {
    const [isNewBlock, setIsNewBlock] = useState(false);
    const prevBlockRef = useRef<bigint | null>(null);

    useEffect(() => {
        if (blockNumber !== null && prevBlockRef.current !== null && blockNumber !== prevBlockRef.current) {
            setIsNewBlock(true);
            const timeout = setTimeout(() => setIsNewBlock(false), 1000);
            prevBlockRef.current = blockNumber;
            return () => clearTimeout(timeout);
        }
        prevBlockRef.current = blockNumber;
    }, [blockNumber]);

    let statusText = 'Connecting to Ethereum...';
    let statusColor = 'bg-app-accent-orange';

    if (isConnected) {
        statusText = blockNumber !== null ? `Block ${String(blockNumber)}` : 'Connected';
        statusColor = isNewBlock ? 'bg-app-accent-blue' : 'bg-app-accent-green';
    } else if (!isLoading) {
        statusText = 'Reconnecting...';
        statusColor = 'bg-app-accent-orange animate-pulse';
    }

    return (
        <div className="flex items-center gap-2 text-sm">
            <div className={`w-2 h-2 rounded-full transition-colors duration-300 ${statusColor}`} />
            <span className={`text-app-text-secondary transition-colors duration-300 ${isNewBlock ? 'text-app-text-primary' : ''}`}>
                {statusText}
            </span>
        </div>
    );
}
