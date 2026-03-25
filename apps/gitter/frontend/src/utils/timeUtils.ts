/**
 * Convert Unix timestamp (seconds) to relative time string (e.g., "2 hours ago")
 */
export function formatRelativeTime(timestamp: number): string {
    const now = Math.floor(Date.now() / 1000); // Current time in seconds
    const secondsAgo = now - timestamp;

    if (secondsAgo < 0) {
        return 'just now';
    }

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

/**
 * Convert Unix timestamp (seconds) to formatted date string
 * Format: "Jan 15, 2024" or custom format
 */
export function formatDate(timestamp: number, includeTime: boolean = false): string {
    const date = new Date(timestamp * 1000);

    const options: Intl.DateTimeFormatOptions = {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
    };

    if (includeTime) {
        options.hour = '2-digit';
        options.minute = '2-digit';
    }

    return date.toLocaleDateString('en-US', options);
}

/**
 * Get current Unix timestamp in seconds
 */
export function getCurrentTimestamp(): number {
    return Math.floor(Date.now() / 1000);
}

/**
 * Create timestamp for relative time (e.g., "2 hours ago" -> timestamp)
 * Useful for generating test data
 */
export function getTimestampFromRelative(value: number, unit: 'seconds' | 'minutes' | 'hours' | 'days' | 'weeks' | 'months'): number {
    const now = getCurrentTimestamp();
    const multipliers = {
        seconds: 1,
        minutes: 60,
        hours: 3600,
        days: 86400,
        weeks: 604800,
        months: 2592000,
    };

    return now - (value * multipliers[unit]);
}
