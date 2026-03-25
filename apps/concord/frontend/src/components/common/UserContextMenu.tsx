import React, { useEffect, useRef } from 'react';
import { createPortal } from 'react-dom';
import { useNavigate } from 'react-router-dom';
import { MessageCircle } from 'lucide-react';

interface UserContextMenuProps {
    x: number;
    y: number;
    targetPubkey: string;
    myPubkey: string;
    onClose: () => void;
}

function UserContextMenu({ x, y, targetPubkey, myPubkey, onClose }: UserContextMenuProps): React.JSX.Element | null {
    const navigate = useNavigate();
    const ref = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handle = (e: MouseEvent) => {
            if (ref.current && !ref.current.contains(e.target as Node)) onClose();
        };
        const handleKey = (e: KeyboardEvent) => {
            if (e.key === 'Escape') onClose();
        };
        document.addEventListener('mousedown', handle);
        document.addEventListener('keydown', handleKey);
        return () => {
            document.removeEventListener('mousedown', handle);
            document.removeEventListener('keydown', handleKey);
        };
    }, [onClose]);

    // Clamp position so the menu doesn't overflow the viewport
    const menuWidth = 180;
    const menuHeight = 40;
    const clampedX = Math.min(x, window.innerWidth - menuWidth - 8);
    const clampedY = Math.min(y, window.innerHeight - menuHeight - 8);

    if (targetPubkey === myPubkey) return null;

    return createPortal(
        <div
            ref={ref}
            className="fixed z-50 bg-dc-bg-floating rounded-md shadow-lg border border-dc-bg-tertiary p-1.5 min-w-[170px]"
            style={{ left: clampedX, top: clampedY }}
        >
            <button
                onClick={() => {
                    navigate(`/dms/${targetPubkey}`);
                    onClose();
                }}
                className="w-full flex items-center gap-2 px-3 py-1.5 text-sm text-dc-text hover:bg-dc-blurple hover:text-white transition-colors rounded-sm mx-0"
            >
                <MessageCircle size={16} />
                Message
            </button>
        </div>,
        document.body,
    );
}

export default UserContextMenu;
