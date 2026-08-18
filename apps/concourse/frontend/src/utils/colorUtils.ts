const AVATAR_COLORS = [
    '#5865f2',
    '#57f287',
    '#eb459e',
    '#ed4245',
    '#3ba55c',
    '#faa61a',
    '#e67e22',
    '#9b59b6',
    '#1abc9c',
    '#2d7d9a',
    '#c0392b',
    '#8e44ad',
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

export function stringToColor(str: string): string {
    const index = hashString(str) % AVATAR_COLORS.length;
    return AVATAR_COLORS[index];
}
