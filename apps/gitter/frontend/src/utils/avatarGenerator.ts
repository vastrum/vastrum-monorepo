// Predefined gradient combinations for avatars
const gradients = [
    'from-purple-600 to-purple-800',
    'from-blue-400 to-cyan-400',
    'from-green-400 to-teal-400',
    'from-yellow-400 to-orange-400',
    'from-pink-500 to-red-500',
    'from-indigo-500 to-purple-500',
    'from-red-500 to-pink-500',
    'from-teal-400 to-green-400',
    'from-orange-400 to-red-400',
    'from-cyan-400 to-blue-400',
];

// Simple hash function to generate consistent index from string
function hashString(str: string): number {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
        const char = str.charCodeAt(i);
        hash = ((hash << 5) - hash) + char;
        hash = hash & hash; // Convert to 32-bit integer
    }
    return Math.abs(hash);
}

/**
 * Generates a consistent avatar gradient based on author name
 * @param author - The author's username
 * @returns A Tailwind gradient class string
 */
export function generateAvatarGradient(author: string): string {
    const index = hashString(author) % gradients.length;
    return gradients[index];
}

export function truncateAddress(address: string, startChars: number = 6, endChars: number = 6): string {
    if (address.length <= startChars + endChars + 3) {
        return address;
    }
    return `${address.slice(0, startChars)}...${address.slice(-endChars)}`;
}
