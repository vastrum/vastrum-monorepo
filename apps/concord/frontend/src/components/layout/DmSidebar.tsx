import React from 'react';
import { useNavigate } from 'react-router-dom';
import { Plus, MessageCircle } from 'lucide-react';
import Avatar from '@/components/common/Avatar';
import EmptyState from '@/components/common/EmptyState';
import UserPanel from '@/components/layout/UserPanel';
import { truncateAddress } from '@/utils/avatarGenerator';
import { formatRelativeTime } from '@/utils/timeUtils';
import { useUnread } from '@/context/UnreadProvider';
import type { JSDmSummary } from '../../../wasm/pkg';

interface DmSidebarProps {
    dms: JSDmSummary[];
    names: Record<string, string>;
    activeDmKey?: string;
    onDmClick: (dm: JSDmSummary) => void;
    onNewMessage: () => void;
    onNavigate?: () => void;
}

function DmSidebar({ dms, names, activeDmKey, onDmClick, onNewMessage, onNavigate }: DmSidebarProps): React.JSX.Element {
    const navigate = useNavigate();
    const { isDmUnread, markDmRead } = useUnread();

    return (
        <div className="w-60 bg-dc-bg-secondary flex flex-col flex-shrink-0">
            <div className="h-12 px-4 flex items-center border-b border-dc-bg-tertiary shadow-sm">
                <span className="font-semibold text-white">Direct Messages</span>
            </div>

            <div className="flex-1 overflow-y-auto p-2">
                <button
                    onClick={onNewMessage}
                    className="w-full flex items-center gap-2 px-2 py-2 text-sm text-dc-text-muted hover:text-dc-text hover:bg-dc-channel-hover rounded transition-colors mb-2"
                >
                    <Plus size={16} />
                    New Message
                </button>

                {dms.length === 0 && (
                    <EmptyState icon={<MessageCircle size={24} className="mx-auto mb-2 opacity-40" />} message="No conversations yet" />
                )}

                {dms.map(dm => {
                    const isActive = dm.other_user === activeDmKey;
                    const unread = !isActive && isDmUnread(dm.other_user);
                    return (
                        <button
                            key={dm.other_user}
                            onClick={() => {
                                markDmRead(dm.other_user);
                                onDmClick(dm);
                                navigate(`/dms/${dm.other_user}`);
                                onNavigate?.();
                            }}
                            className={`w-full flex items-center gap-2 px-2 py-2 rounded transition-colors ${
                                isActive ? 'bg-dc-channel-active' : 'hover:bg-dc-channel-hover'
                            }`}
                        >
                            {unread && <span className="w-2 h-2 rounded-full bg-red-500 flex-shrink-0" />}
                            <Avatar identifier={dm.other_user} name={names[dm.other_user] || dm.other_user} size={32} />
                            <div className="flex-1 min-w-0 text-left">
                                <div className={`text-sm truncate ${unread ? 'text-white font-bold' : 'text-dc-text'}`}>
                                    {names[dm.other_user] || truncateAddress(dm.other_user)}
                                </div>
                                {dm.last_message && (
                                    <div className="text-xs text-dc-text-muted truncate">{dm.last_message}</div>
                                )}
                            </div>
                            {dm.last_timestamp && (
                                <span className="text-xs text-dc-text-muted flex-shrink-0">
                                    {formatRelativeTime(Number(dm.last_timestamp))}
                                </span>
                            )}
                        </button>
                    );
                })}
            </div>
            <UserPanel />
        </div>
    );
}

export default DmSidebar;
