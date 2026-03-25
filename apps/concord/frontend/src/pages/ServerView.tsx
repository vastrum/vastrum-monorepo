import React, { useEffect, useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import {
    get_server, get_messages, get_my_pubkey,
    send_message, delete_message,
    type JSServerDetail, type JSMessage, type JSChannel,
} from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import ChannelSidebar from '@/components/layout/ChannelSidebar';
import MemberSidebar from '@/components/layout/MemberSidebar';
import MessageFeed from '@/components/chat/MessageFeed';
import MessageInput from '@/components/chat/MessageInput';
import { useUnread } from '@/context/UnreadProvider';
import { useMobileSidebar } from '@/context/MobileSidebarProvider';
import { Hash, Menu, Users } from 'lucide-react';

function ServerView(): React.JSX.Element {
    const { serverId, channelId } = useParams();
    const navigate = useNavigate();
    const [server, setServer] = useState<JSServerDetail | null>(null);
    const [messages, setMessages] = useState<JSMessage[]>([]);
    const [myPubkey, setMyPubkey] = useState('');
    const [activeChannelId, setActiveChannelId] = useState<number | null>(null);
    const [activeChannel, setActiveChannel] = useState<JSChannel | null>(null);
    const [loading, setLoading] = useState(true);
    const { markChannelRead } = useUnread();
    const { sidebarOpen, openSidebar, closeSidebar } = useMobileSidebar();
    const [membersOpen, setMembersOpen] = useState(false);

    const sid = serverId ? Number(serverId) : null;

    const loadServer = useCallback(async () => {
        if (sid === null) return;
        const [s, pk] = await Promise.all([
            get_server(BigInt(sid)),
            get_my_pubkey(),
        ]);
        if (!s) {
            navigate('/dms', { replace: true });
            setLoading(false);
            return;
        }
        setServer(s);
        setMyPubkey(pk);

        // Navigate to first channel if none selected
        if (!channelId && s && s.channels.length > 0) {
            navigate(`/server/${serverId}/${s.channels[0].id}`, { replace: true });
        }
        setLoading(false);
    }, [sid, channelId, navigate, serverId]);

    useEffect(() => { loadServer(); }, [loadServer]);

    // Poll server (~10s)
    useEffect(() => {
        if (sid === null) return;
        const poll = async () => {
            const s = await get_server(BigInt(sid));
            if (s) setServer(s);
        };
        const interval = setInterval(poll, 10_000);
        return () => clearInterval(interval);
    }, [sid]);

    // Load messages when channel changes + mark channel as read
    useEffect(() => {
        if (!channelId || sid === null) return;
        const cid = Number(channelId);
        setActiveChannelId(cid);
        const ch = server?.channels.find(c => String(c.id) === channelId);
        setActiveChannel(ch ?? null);

        if (ch) markChannelRead(sid, ch.id);

        const loadMessages = async () => {
            const msgs = await get_messages(BigInt(sid), BigInt(cid), BigInt(200));
            setMessages(msgs);
        };
        loadMessages();

        const interval = setInterval(loadMessages, 3000);
        return () => clearInterval(interval);
    }, [channelId, server, sid]);

    const handleSendMessage = async (content: string) => {
        if (activeChannelId === null || sid === null) return;
        const txHash = await send_message(BigInt(sid), BigInt(activeChannelId), content);
        if (txHash) await await_tx_inclusion(txHash);
        const [msgs, s] = await Promise.all([
            get_messages(BigInt(sid), BigInt(activeChannelId), BigInt(200)),
            get_server(BigInt(sid)),
        ]);
        setMessages(msgs);
        if (s) setServer(s);
        const ch = s?.channels.find(c => String(c.id) === String(activeChannelId));
        if (ch) markChannelRead(sid, ch.id);
    };

    const handleDeleteMessage = async (messageId: number) => {
        if (activeChannelId === null || sid === null) return;
        const txHash = await delete_message(BigInt(sid), BigInt(activeChannelId), BigInt(messageId));
        await await_tx_inclusion(txHash);
        const msgs = await get_messages(BigInt(sid), BigInt(activeChannelId), BigInt(200));
        setMessages(msgs);
    };

    const memberNames: Record<string, string> = {};
    if (server) {
        for (const m of server.members) {
            if (m.display_name) memberNames[m.pubkey] = m.display_name;
        }
    }

    if (loading) {
        return <div className="flex-1 flex items-center justify-center text-dc-text-muted">Loading server...</div>;
    }

    if (!server) {
        return <div className="flex-1 flex items-center justify-center text-dc-text-muted">Server not found</div>;
    }

    return (
        <>
            <div className={`fixed inset-y-0 left-0 z-50 flex transition-transform duration-200 ${sidebarOpen ? 'translate-x-[72px]' : '-translate-x-full'} md:relative md:translate-x-0 md:z-auto md:transition-none`}>
                <ChannelSidebar
                    server={server}
                    activeChannelId={activeChannelId}
                    myPubkey={myPubkey}
                    onRefresh={loadServer}
                    onNavigate={closeSidebar}
                />
            </div>

            {/* Main chat area */}
            <div className="flex-1 flex flex-col bg-dc-bg-primary min-w-0">
                {/* Channel header */}
                <div className="h-12 flex items-center px-4 border-b border-dc-bg-tertiary shadow-sm flex-shrink-0">
                    <button onClick={openSidebar} className="mr-2 text-dc-text-muted hover:text-dc-text md:hidden">
                        <Menu size={20} />
                    </button>
                    <Hash size={20} className="text-dc-text-muted mr-1" />
                    <span className="font-semibold text-white flex-1">{activeChannel?.name || 'general'}</span>
                    <button onClick={() => setMembersOpen(true)} className="ml-2 text-dc-text-muted hover:text-dc-text lg:hidden">
                        <Users size={20} />
                    </button>
                </div>

                <MessageFeed
                    messages={messages}
                    memberNames={memberNames}
                    myPubkey={myPubkey}
                    ownerPubkey={server.owner}
                    onDeleteMessage={handleDeleteMessage}
                />

                <MessageInput
                    placeholder={`Message #${activeChannel?.name || 'general'}`}
                    onSend={handleSendMessage}
                />
            </div>

            {membersOpen && (
                <div className="fixed inset-0 bg-black/50 z-40 lg:hidden" onClick={() => setMembersOpen(false)} />
            )}
            <div className={`fixed inset-y-0 right-0 z-50 flex transition-transform duration-200 ${membersOpen ? 'translate-x-0' : 'translate-x-full'} lg:relative lg:translate-x-0 lg:z-auto lg:transition-none`}>
                <MemberSidebar members={server.members} myPubkey={myPubkey} />
            </div>
        </>
    );
}

export default ServerView;
