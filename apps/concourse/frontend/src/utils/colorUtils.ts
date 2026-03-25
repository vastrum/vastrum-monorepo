const AVATAR_COLORS = [
    '#5865f2', // blurple
    '#57f287', // green
    '#eb459e', // fuchsia
    '#ed4245', // red
    '#3ba55c', // dark green
    '#faa61a', // orange
    '#e67e22', // dark orange
    '#9b59b6', // purple
    '#1abc9c', // teal
    '#2d7d9a', // steel blue
    '#c0392b', // crimson
    '#8e44ad', // deep purple
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
