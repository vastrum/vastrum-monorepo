import React, { useState, useCallback } from 'react';
import Avatar from '@/components/common/Avatar';
import UserContextMenu from '@/components/common/UserContextMenu';
import { truncateAddress } from '@/utils/avatarGenerator';

interface MemberInfo {
    pubkey: string;
    display_name: string;
}

interface MemberSidebarProps {
    members: MemberInfo[];
    myPubkey: string;
}

function MemberSidebar({ members, myPubkey }: MemberSidebarProps): React.JSX.Element {
    const [ctxMenu, setCtxMenu] = useState<{ x: number; y: number; pubkey: string } | null>(null);

    const handleContextMenu = useCallback((e: React.MouseEvent, pubkey: string) => {
        e.preventDefault();
        setCtxMenu({ x: e.clientX, y: e.clientY, pubkey });
    }, []);

    return (
        <div className="w-60 bg-dc-bg-secondary flex-shrink-0 overflow-y-auto">
            <div className="p-4">
                <h3 className="text-xs font-semibold uppercase tracking-wide mb-2 text-dc-text-muted">
                    Members - {members.length}
                </h3>
                <div className="space-y-0.5">
                    {members.map(m => (
                        <div
                            key={m.pubkey}
                            onClick={e => handleContextMenu(e, m.pubkey)}
                            onContextMenu={e => handleContextMenu(e, m.pubkey)}
                            className="flex items-center gap-2 px-2 py-1.5 rounded hover:bg-dc-channel-hover transition-colors cursor-pointer"
                        >
                            <Avatar identifier={m.pubkey} name={m.display_name || m.pubkey} size={32} />
                            <span className="text-sm text-dc-text truncate">
                                {m.display_name || truncateAddress(m.pubkey)}
                            </span>
                        </div>
                    ))}
                </div>
            </div>
            {ctxMenu && (
                <UserContextMenu
                    x={ctxMenu.x}
                    y={ctxMenu.y}
                    targetPubkey={ctxMenu.pubkey}
                    myPubkey={myPubkey}
                    onClose={() => setCtxMenu(null)}
                />
            )}
        </div>
    );
}

export default MemberSidebar;
