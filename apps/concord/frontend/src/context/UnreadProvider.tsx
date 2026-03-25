import { createContext, useContext, useEffect, useState, useCallback, type ReactNode } from 'react';
import { get_my_dms, get_my_servers, get_channels } from '../../wasm/pkg';

interface UnreadState {
    markDmRead: (dmKey: string) => void;
    markChannelRead: (serverId: number, channelId: number) => void;
    isDmUnread: (dmKey: string) => boolean;
    isChannelUnread: (serverId: number, channelId: number) => boolean;
    hasAnyDmUnread: () => boolean;
    hasServerUnread: (serverId: number) => boolean;
}

const UnreadContext = createContext<UnreadState>(null!);

export function useUnread() {
    return useContext(UnreadContext);
}

export function UnreadProvider({ children }: { children: ReactNode }) {
    const [baselines, setBaselines] = useState<Record<string, number>>({});
    const [currentIds, setCurrentIds] = useState<Record<string, number>>({});

    useEffect(() => {
        const poll = async () => {
            const newIds: Record<string, number> = {};

            const [dms, servers] = await Promise.all([
                get_my_dms(BigInt(200), BigInt(0)).catch(() => [] as any[]),
                get_my_servers().catch(() => [] as any[]),
            ]);

            for (const dm of dms) newIds[`dm:${dm.other_user}`] = Number(dm.next_message_id);

            const channelResults = await Promise.all(
                servers.map(s => get_channels(BigInt(s.id)).catch(() => [] as any[]))
            );
            for (let i = 0; i < servers.length; i++) {
                for (const ch of channelResults[i]) {
                    newIds[`ch:${servers[i].id}:${ch.id}`] = Number(ch.next_message_id);
                }
            }

            setCurrentIds(newIds);
        };

        poll();
        const interval = setInterval(poll, 3_000);
        return () => clearInterval(interval);
    }, []);

    const markRead = useCallback((key: string) => {
        setBaselines(prev => ({ ...prev, [key]: currentIds[key] ?? prev[key] }));
    }, [currentIds]);

    const isUnread = useCallback((key: string) => {
        return (currentIds[key] ?? 0) > (baselines[key] ?? 0);
    }, [currentIds, baselines]);

    const markDmRead = useCallback((dmKey: string) => markRead(`dm:${dmKey}`), [markRead]);
    const markChannelRead = useCallback((serverId: number, channelId: number) => markRead(`ch:${serverId}:${channelId}`), [markRead]);
    const isDmUnread = useCallback((dmKey: string) => isUnread(`dm:${dmKey}`), [isUnread]);
    const isChannelUnread = useCallback((serverId: number, channelId: number) => isUnread(`ch:${serverId}:${channelId}`), [isUnread]);

    const hasAnyDmUnread = useCallback(() => {
        return Object.keys(currentIds).some(k => k.startsWith('dm:') && isUnread(k));
    }, [currentIds, isUnread]);

    const hasServerUnread = useCallback((serverId: number) => {
        return Object.keys(currentIds).some(k => k.startsWith(`ch:${serverId}:`) && isUnread(k));
    }, [currentIds, isUnread]);

    return (
        <UnreadContext.Provider value={{ markDmRead, markChannelRead, isDmUnread, isChannelUnread, hasAnyDmUnread, hasServerUnread }}>
            {children}
        </UnreadContext.Provider>
    );
}
