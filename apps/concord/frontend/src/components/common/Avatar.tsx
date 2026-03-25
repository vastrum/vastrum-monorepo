import React from 'react';
import { getAvatarColor, getInitials } from '@/utils/avatarGenerator';

interface AvatarProps {
    identifier: string;
    name?: string;
    size?: number;
}

function Avatar({ identifier, name, size = 40 }: AvatarProps): React.JSX.Element {
    const color = getAvatarColor(identifier);
    const initials = getInitials(name || identifier);

    return (
        <div
            className="rounded-full flex items-center justify-center flex-shrink-0 font-semibold text-white select-none"
            style={{
                backgroundColor: color,
                width: size,
                height: size,
                fontSize: size * 0.4,
            }}
        >
            {initials}
        </div>
    );
}

export default Avatar;
