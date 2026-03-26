const AVATAR_COLORS = [
    '#5865f2', // blurple
    '#57f287', // green
    '#fee75c', // yellow
    '#eb459e', // fuchsia
    '#ed4245', // red
    '#3ba55c', // dark green
    '#faa61a', // orange
    '#e67e22', // dark orange
    '#9b59b6', // purple
    '#1abc9c', // teal
];

function hashString(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        const char = str.charCodeAt(i);
        hash = ((hash << 5) - hash) + char;
        hash = hash & hash;
    }
    return Math.abs(hash);
}

export function getAvatarColor(identifier: string): string {
    const index = hashString(identifier) % AVATAR_COLORS.length;
    return AVATAR_COLORS[index];
}

export function getInitials(name: string): string {
    if (!name) return '?';
    const parts = name.trim().split(/\s+/);
    if (parts.length >= 2) {
        return (parts[0][0] + parts[1][0]).toUpperCase();
    }
    return name.slice(0, 2).toUpperCase();
}

export function truncateAddress(address: string, startChars: number = 6, endChars: number = 6): string {
    if (address.length <= startChars + endChars + 3) {
        return address;
    }
    return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
}
