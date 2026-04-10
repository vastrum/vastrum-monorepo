export function validateSshKey(key: string): string | null {
    const parts = key.trim().split(/\s+/);
    if (parts.length < 2) {
        return 'Invalid SSH public key. Expected format: ssh-ed25519 AAAA... user@host';
    }
    const validTypes = [
        'ssh-ed25519',
        'ssh-rsa',
        'ecdsa-sha2-nistp256',
        'ecdsa-sha2-nistp384',
        'ecdsa-sha2-nistp521',
    ];
    if (!validTypes.includes(parts[0])) {
        return `Unsupported key type "${parts[0]}". Use ssh-ed25519, ssh-rsa, or ecdsa-sha2-*`;
    }
    try {
        atob(parts[1]);
    } catch {
        return 'Invalid base64 in SSH key';
    }
    return null;
}
