import React, { useState, useCallback } from 'react';
import Avatar from '@/components/common/Avatar';
import MarkdownRenderer from '@/components/common/MarkdownRenderer';
import ConfirmDialog from '@/components/common/ConfirmDialog';
import UserContextMenu from '@/components/common/UserContextMenu';
import { formatMessageTime } from '@/utils/timeUtils';
import { truncateAddress } from '@/utils/avatarGenerator';
import { Trash2 } from 'lucide-react';

interface MessageItemProps {
    id: number;
    content: string;
    author: string;
    timestamp: number;
    authorName?: string;
    myPubkey: string;
    canDelete: boolean;
    onDelete: () => void;
    showHeader: boolean;
}

function MessageItem({ content, author, timestamp, authorName, myPubkey, canDelete, onDelete, showHeader }: MessageItemProps): React.JSX.Element {
    const displayName = authorName || truncateAddress(author);
    const [showConfirm, setShowConfirm] = useState(false);
    const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number } | null>(null);

    const handleAuthorContextMenu = useCallback((e: React.MouseEvent) => {
        e.preventDefault();
        setCtxMenu({ x: e.clientX, y: e.clientY });
    }, []);

    return (
        <>
            {showHeader ? (
                <div className="group flex items-start gap-4 px-4 pt-4 pb-0.5 hover:bg-dc-bg-primary/30 relative">
                    <div onClick={handleAuthorContextMenu} onContextMenu={handleAuthorContextMenu}>
                        <Avatar identifier={author} name={displayName} size={40} />
                    </div>
                    <div className="flex-1 min-w-0">
                        <div className="flex items-baseline gap-2">
                            <span onClick={handleAuthorContextMenu} onContextMenu={handleAuthorContextMenu} className="font-medium text-white hover:underline cursor-pointer">{displayName}</span>
                            <span className="text-xs text-dc-text-muted">{formatMessageTime(Number(timestamp))}</span>
                        </div>
                        <div className="text-sm text-dc-text">
                            <MarkdownRenderer content={content} />
                        </div>
                    </div>
                    {canDelete && (
                        <button
                            onClick={() => setShowConfirm(true)}
                            className="absolute right-4 top-2 opacity-0 group-hover:opacity-100 text-dc-text-muted hover:text-dc-red transition-all p-1"
                        >
                            <Trash2 size={14} />
                        </button>
                    )}
                </div>
            ) : (
                <div className="group flex items-start gap-4 px-4 py-0.5 hover:bg-dc-bg-primary/30 relative">
                    <div className="w-10 flex-shrink-0" />
                    <div className="flex-1 min-w-0">
                        <div className="text-sm text-dc-text">
                            <MarkdownRenderer content={content} />
                        </div>
                    </div>
                    {canDelete && (
                        <button
                            onClick={() => setShowConfirm(true)}
                            className="absolute right-4 top-1 opacity-0 group-hover:opacity-100 text-dc-text-muted hover:text-dc-red transition-all p-1"
                        >
                            <Trash2 size={14} />
                        </button>
                    )}
                </div>
            )}
            <ConfirmDialog
                isOpen={showConfirm}
                onClose={() => setShowConfirm(false)}
                onConfirm={() => { onDelete(); setShowConfirm(false); }}
                title="Delete Message"
                message="Are you sure you want to delete this message?"
                confirmLabel="Delete"
                danger
            />
            {ctxMenu && (
                <UserContextMenu
                    x={ctxMenu.x}
                    y={ctxMenu.y}
                    targetPubkey={author}
                    myPubkey={myPubkey}
                    onClose={() => setCtxMenu(null)}
                />
            )}
        </>
    );
}

export default MessageItem;
