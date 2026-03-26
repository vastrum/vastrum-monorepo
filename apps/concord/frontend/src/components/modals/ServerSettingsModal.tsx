import React, { useState, type ReactNode } from 'react';
import { useNavigate } from 'react-router-dom';
import Modal from '@/components/common/Modal';
import ConfirmDialog from '@/components/common/ConfirmDialog';
import Avatar from '@/components/common/Avatar';
import { truncateAddress } from '@/utils/avatarGenerator';
import { kick_member, leave_server, delete_channel } from '../../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import { LogOut, UserMinus, Shield, Hash, Trash2 } from 'lucide-react';

interface MemberInfo {
    pubkey: string;
    display_name: string;
}

interface ServerDetail {
    id: number;
    name: string;
    owner: string;
    members: MemberInfo[];
    channels: Array<{ id: number; name: string; message_count: number; next_message_id: number }>;
}

interface ServerSettingsModalProps {
    isOpen: boolean;
    onClose: () => void;
    server: ServerDetail;
    myPubkey: string;
    onRefresh: () => void;
}

function ServerSettingsModal({ isOpen, onClose, server, myPubkey, onRefresh }: ServerSettingsModalProps): React.JSX.Element {
    const navigate = useNavigate();
    const [tab, setTab] = useState<'members' | 'channels'>('members');
    const [loading, setLoading] = useState(false);
    const [confirmAction, setConfirmAction] = useState<null | { title: string; message: ReactNode; danger: boolean; confirmLabel: string; action: () => Promise<void> }>(null);

    const isOwner = myPubkey === server.owner;

    const handleKick = async (target: string) => {
        setLoading(true);
        const txHash = await kick_member(BigInt(server.id), target);
        await await_tx_inclusion(txHash);
        setLoading(false);
        onRefresh();
    };

    const handleLeave = async () => {
        setLoading(true);
        const txHash = await leave_server(BigInt(server.id));
        await await_tx_inclusion(txHash);
        setLoading(false);
        onClose();
        navigate('/dms');
    };

    const handleDeleteChannel = async (channelId: number) => {
        if (server.channels.length <= 1) return;
        setLoading(true);
        const txHash = await delete_channel(BigInt(server.id), BigInt(channelId));
        await await_tx_inclusion(txHash);
        setLoading(false);
        onRefresh();
    };

    return (
        <Modal isOpen={isOpen} onClose={onClose} title={`${server.name} - Settings`}>
            <div className="p-4">
                {/* Tabs */}
                <div className="flex gap-2 mb-4 border-b border-dc-border pb-2">
                    <button
                        onClick={() => setTab('members')}
                        className={`px-3 py-1.5 rounded text-sm transition-colors ${tab === 'members' ? 'bg-dc-channel-active text-white' : 'text-dc-text-muted hover:text-dc-text'}`}
                    >
                        Members ({server.members.length})
                    </button>
                    <button
                        onClick={() => setTab('channels')}
                        className={`px-3 py-1.5 rounded text-sm transition-colors ${tab === 'channels' ? 'bg-dc-channel-active text-white' : 'text-dc-text-muted hover:text-dc-text'}`}
                    >
                        Channels ({server.channels.length})
                    </button>
                </div>

                {tab === 'members' && (
                    <div className="space-y-2 max-h-80 overflow-y-auto">
                        {server.members.map(m => (
                            <div key={m.pubkey} className="flex items-center gap-3 p-2 rounded hover:bg-dc-channel-hover">
                                <Avatar identifier={m.pubkey} name={m.display_name || m.pubkey} size={32} />
                                <div className="flex-1 min-w-0">
                                    <div className="text-sm text-dc-text truncate">{m.display_name || truncateAddress(m.pubkey)}</div>
                                </div>
                                {isOwner && m.pubkey !== myPubkey && (
                                    <div className="flex gap-1">
                                        <button
                                            onClick={() => setConfirmAction({
                                                title: 'Kick Member',
                                                message: <>Remove <strong>{m.display_name || truncateAddress(m.pubkey)}</strong> from this server?</>,
                                                danger: true,
                                                confirmLabel: 'Kick',
                                                action: () => handleKick(m.pubkey),
                                            })}
                                            disabled={loading}
                                            className="text-dc-text-muted hover:text-dc-red p-1 transition-colors"
                                            title="Kick"
                                        >
                                            <UserMinus size={14} />
                                        </button>
                                    </div>
                                )}
                                {m.pubkey === server.owner && (
                                    <Shield size={14} className="text-yellow-400 flex-shrink-0" />
                                )}
                            </div>
                        ))}
                    </div>
                )}

                {tab === 'channels' && (
                    <div className="space-y-2 max-h-80 overflow-y-auto">
                        {server.channels.map(ch => (
                            <div key={String(ch.id)} className="flex items-center gap-2 p-2 rounded hover:bg-dc-channel-hover">
                                <Hash size={16} className="text-dc-text-muted" />
                                <span className="text-sm text-dc-text flex-1">{ch.name}</span>
                                {isOwner && server.channels.length > 1 && (
                                    <button
                                        onClick={() => {
                                            setConfirmAction({
                                                title: 'Delete Channel',
                                                message: <>Delete <strong>#{ch.name}</strong>? This cannot be undone.</>,
                                                danger: true,
                                                confirmLabel: 'Delete',
                                                action: () => handleDeleteChannel(ch.id),
                                            });
                                        }}
                                        disabled={loading}
                                        className="text-dc-text-muted hover:text-dc-red p-1 transition-colors"
                                    >
                                        <Trash2 size={14} />
                                    </button>
                                )}
                            </div>
                        ))}
                    </div>
                )}

                {/* Leave button */}
                {!isOwner && (
                    <div className="mt-4 pt-4 border-t border-dc-border">
                        <button
                            onClick={() => setConfirmAction({
                                title: 'Leave Server',
                                message: <>Leave <strong>{server.name}</strong>? You'll need a new invite to rejoin.</>,
                                danger: true,
                                confirmLabel: 'Leave Server',
                                action: handleLeave,
                            })}
                            disabled={loading}
                            className="flex items-center gap-2 text-dc-red hover:bg-dc-red/10 px-3 py-2 rounded text-sm transition-colors w-full"
                        >
                            <LogOut size={16} />
                            Leave Server
                        </button>
                    </div>
                )}
            </div>
            <ConfirmDialog
                isOpen={confirmAction !== null}
                onClose={() => setConfirmAction(null)}
                onConfirm={async () => {
                    if (confirmAction) {
                        try {
                            await confirmAction.action();
                        } finally {
                            setConfirmAction(null);
                        }
                    }
                }}
                title={confirmAction?.title ?? ''}
                message={confirmAction?.message ?? ''}
                confirmLabel={confirmAction?.confirmLabel}
                danger={confirmAction?.danger}
            />
        </Modal>
    );
}

export default ServerSettingsModal;
