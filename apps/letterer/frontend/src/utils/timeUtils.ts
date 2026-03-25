export function formatRelativeTime(timestamp: number): string {
    const now = Math.floor(Date.now() / 1000);
    const secondsAgo = now - timestamp;

    if (secondsAgo < 0) return 'just now';

    const intervals = {
        year: 31536000,
        month: 2592000,
        week: 604800,
        day: 86400,
        hour: 3600,
        minute: 60,
    };

    for (const [unit, seconds] of Object.entries(intervals)) {
        const interval = Math.floor(secondsAgo / seconds);
        if (interval >= 1) {
            return `${interval} ${unit}${interval === 1 ? '' : 's'} ago`;
        }
    }

    return 'just now';
}
