export function truncateHash(hash: string, chars: number = 8): string {
    if (hash.length <= chars * 2 + 3) return hash;
    return `${hash.slice(0, chars)}...${hash.slice(-chars)}`;
}

export function formatTimestamp(timestamp: number): string {
    if (timestamp === 0) return '-';
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
}

export function formatRelativeTime(timestamp: number): string {
    if (timestamp === 0) return '-';
    const now = Math.floor(Date.now() / 1000);
    const diff = now - timestamp;

    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return `${Math.floor(diff / 86400)}d ago`;
}

export function txTypeColor(txType: string): string {
    switch (txType) {
        case 'Call': return 'text-blue-400';
        case 'DeployNewModule': return 'text-emerald-400';
        case 'DeployStoredModule': return 'text-green-400';
        case 'RegisterDomain': return 'text-amber-400';
        case 'AddModule': return 'text-cyan-400';
        default: return 'text-blocker-text-secondary';
    }
}
