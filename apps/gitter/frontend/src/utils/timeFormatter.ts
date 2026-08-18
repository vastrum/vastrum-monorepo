export function formatTimeAgo(timestamp: number): string {
    const now = Math.floor(Date.now() / 1000);
    const diff = now - timestamp;

    if (diff < 60) {
        return 'just now';
    }

    if (diff < 3600) {
        const minutes = Math.floor(diff / 60);
        return `${minutes} ${minutes === 1 ? 'minute' : 'minutes'} ago`;
    }

    if (diff < 86400) {
        const hours = Math.floor(diff / 3600);
        return `${hours} ${hours === 1 ? 'hour' : 'hours'} ago`;
    }

    if (diff < 604800) {
        const days = Math.floor(diff / 86400);
        return `${days} ${days === 1 ? 'day' : 'days'} ago`;
    }

    if (diff < 2592000) {
        const weeks = Math.floor(diff / 604800);
        return `${weeks} ${weeks === 1 ? 'week' : 'weeks'} ago`;
    }

    if (diff < 31536000) {
        const months = Math.floor(diff / 2592000);
        return `${months} ${months === 1 ? 'month' : 'months'} ago`;
    }

    const years = Math.floor(diff / 31536000);
    return `${years} ${years === 1 ? 'year' : 'years'} ago`;
}

export function formatDate(
    timestamp: number,
    options?: Intl.DateTimeFormatOptions
): string {
    const date = new Date(timestamp * 1000);
    const defaultOptions: Intl.DateTimeFormatOptions = {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        ...options,
    };
    return new Intl.DateTimeFormat('en-US', defaultOptions).format(date);
}

export function formatDateTime(timestamp: number): string {
    const date = new Date(timestamp * 1000);
    const options: Intl.DateTimeFormatOptions = {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        timeZoneName: 'short',
    };
    return new Intl.DateTimeFormat('en-US', options).format(date);
}

export function getCurrentTimestamp(): number {
    return Math.floor(Date.now() / 1000);
}

export function getTimestampFromOffset(offset: {
    seconds?: number;
    minutes?: number;
    hours?: number;
    days?: number;
    weeks?: number;
    months?: number;
    years?: number;
}): number {
    const now = getCurrentTimestamp();
    const totalSeconds =
        (offset.seconds || 0) +
        (offset.minutes || 0) * 60 +
        (offset.hours || 0) * 3600 +
        (offset.days || 0) * 86400 +
        (offset.weeks || 0) * 604800 +
        (offset.months || 0) * 2592000 +
        (offset.years || 0) * 31536000;
    return now - totalSeconds;
}
