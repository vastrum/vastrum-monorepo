import React, { useEffect, useState, useCallback } from 'react';
import { useParams } from 'react-router-dom';
import {
    get_dm_messages, get_my_pubkey, get_my_dms, get_user_profile,
    send_dm, type JSMessage, type JSDmSummary,
} from '../../wasm/pkg';
import { await_tx_inclusion } from '@vastrum/react-lib';
import MessageFeed from '@/components/chat/MessageFeed';
import MessageInput from '@/components/chat/MessageInput';
import DmSidebar from '@/components/layout/DmSidebar';
import NewDmModal from '@/components/modals/NewDmModal';
import { truncateAddress } from '@/utils/avatarGenerator';
import { useUnread } from '@/context/UnreadProvider';
import { useMobileSidebar } from '@/context/MobileSidebarProvider';
import { resolveDisplayNames } from '@/utils/resolveDisplayNames';
import { AtSign, Menu } from 'lucide-react';

function DmView(): React.JSX.Element {
    const { dmKey } = useParams();
    const [messages, setMessages] = useState<JSMessage[]>([]);
    const [myPubkey, setMyPubkey] = useState('');
    const [otherUser, setOtherUser] = useState('');
    const [otherName, setOtherName] = useState('');
    const [dms, setDms] = useState<JSDmSummary[]>([]);
    const [names, setNames] = useState<Record<string, string>>({});
    const [showNewDm, setShowNewDm] = useState(false);
    const { markDmRead } = useUnread();
    const { sidebarOpen, openSidebar, closeSidebar } = useMobileSidebar();

    const loadDms = async () => {
        const [myDms, pk] = await Promise.all([get_my_dms(BigInt(200), BigInt(0)), get_my_pubkey()]);
        setDms(myDms);
        setMyPubkey(pk);
        setNames(await resolveDisplayNames(myDms));
    };

    useEffect(() => { loadDms(); }, []);

    // Poll DM list (~10s)
    useEffect(() => {
        const poll = async () => {
            const myDms = await get_my_dms(BigInt(200), BigInt(0));
            setDms(myDms);
            // Keep active DM marked as read
            if (dmKey && otherUser) markDmRead(otherUser);
        };
        const interval = setInterval(poll, 10_000);
        return () => clearInterval(interval);
    }, [dmKey, otherUser]);

    const loadMessages = useCallback(async () => {
        if (!otherUser) return;
        const msgs = await get_dm_messages(otherUser, BigInt(200));
        setMessages(msgs);
    }, [otherUser]);

    // Resolve other user from dmKey + mark as read
    useEffect(() => {
        if (!dmKey || !myPubkey) return;
        const parts = dmKey.split('_');
        const other = parts[0] === myPubkey ? parts[1] : parts[0];
        setOtherUser(other);

        markDmRead(other);

        const resolveName = async () => {
            const profile = await get_user_profile(other);
            if (profile.display_name) setOtherName(profile.display_name);
        };
        resolveName();
    }, [dmKey, myPubkey]);

    useEffect(() => {
        loadMessages();
        const interval = setInterval(loadMessages, 3000);
        return () => clearInterval(interval);
    }, [loadMessages]);

    const handleSend = async (content: string) => {
        if (!otherUser) return;
        const txHash = await send_dm(otherUser, content);
        await await_tx_inclusion(txHash);
        await loadMessages();
    };

    const displayName = otherName || truncateAddress(otherUser);
    const memberNames: Record<string, string> = { ...names };
    if (otherName) memberNames[otherUser] = otherName;

    return (
        <>
            <div className={`fixed inset-y-0 left-0 z-50 flex transition-transform duration-200 ${sidebarOpen ? 'translate-x-[72px]' : '-translate-x-full'} md:relative md:translate-x-0 md:z-auto md:transition-none`}>
                <DmSidebar
                    dms={dms}
                    names={names}
                    activeDmKey={otherUser}
                    onDmClick={() => {}}
                    onNewMessage={() => setShowNewDm(true)}
                    onNavigate={closeSidebar}
                />
            </div>

            {/* Chat area */}
            <div className="flex-1 flex flex-col bg-dc-bg-primary min-w-0">
                <div className="h-12 flex items-center px-4 border-b border-dc-bg-tertiary shadow-sm flex-shrink-0">
                    <button onClick={openSidebar} className="mr-2 text-dc-text-muted hover:text-dc-text md:hidden">
                        <Menu size={20} />
                    </button>
                    <AtSign size={20} className="text-dc-text-muted mr-2" />
                    <span className="font-semibold text-white">{displayName}</span>
                </div>

                <MessageFeed
                    messages={messages}
                    memberNames={memberNames}
                    myPubkey={myPubkey}
                    ownerPubkey=""
                    onDeleteMessage={() => {}}
                />

                <MessageInput
                    placeholder={`Message @${displayName}`}
                    onSend={handleSend}
                />
            </div>

            <NewDmModal
                isOpen={showNewDm}
                onClose={() => setShowNewDm(false)}
                onSent={() => { setShowNewDm(false); loadDms(); loadMessages(); }}
            />
        </>
    );
}

export default DmView;
