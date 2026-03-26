import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Hash, Plus, Settings, ChevronDown } from 'lucide-react';
import UserPanel from '@/components/layout/UserPanel';
import EmptyState from '@/components/common/EmptyState';
import CreateChannelModal from '@/components/modals/CreateChannelModal';
import ServerSettingsModal from '@/components/modals/ServerSettingsModal';
import InviteModal from '@/components/modals/InviteModal';
import { useUnread } from '@/context/UnreadProvider';

interface ServerDetail {
    id: number;
    name: string;
    owner: string;
    members: Array<{ pubkey: string; display_name: string }>;
    channels: Array<{ id: number; name: string; message_count: number; next_message_id: number }>;
}

interface ChannelSidebarProps {
    server: ServerDetail;
    activeChannelId: number | null;
    myPubkey: string;
    onRefresh: () => void;
    onNavigate?: () => void;
}

function ChannelSidebar({ server, activeChannelId, myPubkey, onRefresh, onNavigate }: ChannelSidebarProps): React.JSX.Element {
    const navigate = useNavigate();
    const [showCreateChannel, setShowCreateChannel] = useState(false);
    const [showSettings, setShowSettings] = useState(false);
    const [showInvite, setShowInvite] = useState(false);
    const { isChannelUnread } = useUnread();

    const isOwner = server.owner === myPubkey;

    return (
        <>
            <div className="w-60 bg-dc-bg-secondary flex flex-col flex-shrink-0">
                {/* Server header */}
                <button
                    onClick={() => setShowSettings(true)}
                    className="h-12 px-4 flex items-center justify-between border-b border-dc-bg-tertiary hover:bg-dc-channel-hover transition-colors shadow-sm"
                >
                    <span className="font-semibold text-white truncate">{server.name}</span>
                    <ChevronDown size={16} className="text-dc-text-muted flex-shrink-0" />
                </button>

                {/* Channels */}
                <div className="flex-1 overflow-y-auto pt-4 px-2">
                    <div className="flex items-center justify-between px-1 mb-1">
                        <span className="text-xs font-semibold text-dc-text-muted uppercase tracking-wide">Text Channels</span>
                        {isOwner && (
                            <button
                                onClick={() => setShowCreateChannel(true)}
                                className="text-dc-text-muted hover:text-dc-text transition-colors"
                            >
                                <Plus size={16} />
                            </button>
                        )}
                    </div>

                    {server.channels.length === 0 && (
                        <EmptyState icon={<Hash size={24} className="mx-auto mb-2 opacity-40" />} message="No channels yet" />
                    )}

                    {server.channels.map(ch => {
                        const isActive = activeChannelId !== null && String(ch.id) === String(activeChannelId);
                        const isUnread = !isActive && isChannelUnread(server.id, ch.id);
                        return (
                            <button
                                key={String(ch.id)}
                                onClick={() => { navigate(`/server/${server.id}/${ch.id}`); onNavigate?.(); }}
                                className={`w-full flex items-center gap-1.5 px-2 py-1.5 rounded text-sm transition-colors ${
                                    isActive
                                        ? 'bg-dc-channel-active text-white'
                                        : isUnread
                                            ? 'text-white font-bold hover:bg-dc-channel-hover'
                                            : 'text-dc-text-muted hover:bg-dc-channel-hover hover:text-dc-text'
                                }`}
                            >
                                {isUnread && <span className="w-2 h-2 rounded-full bg-red-500 flex-shrink-0" />}
                                <Hash size={18} className="flex-shrink-0 opacity-60" />
                                <span className="truncate">{ch.name}</span>
                            </button>
                        );
                    })}
                </div>

                {/* Invite button */}
                <div className="p-2 border-t border-dc-bg-tertiary">
                    <button
                        onClick={() => setShowInvite(true)}
                        className="w-full py-1.5 text-sm text-dc-text-muted hover:text-dc-text bg-dc-bg-tertiary hover:bg-dc-channel-hover rounded transition-colors flex items-center justify-center gap-1"
                    >
                        <Settings size={14} />
                        Invite People
                    </button>
                </div>
                <UserPanel />
            </div>

            <CreateChannelModal
                isOpen={showCreateChannel}
                onClose={() => setShowCreateChannel(false)}
                serverId={server.id}
                onCreated={() => { setShowCreateChannel(false); onRefresh(); }}
            />
            <ServerSettingsModal
                isOpen={showSettings}
                onClose={() => setShowSettings(false)}
                server={server}
                myPubkey={myPubkey}
                onRefresh={onRefresh}
            />
            <InviteModal
                isOpen={showInvite}
                onClose={() => setShowInvite(false)}
                serverId={server.id}
                serverName={server.name}
            />
        </>
    );
}

export default ChannelSidebar;
