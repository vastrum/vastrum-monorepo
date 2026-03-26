import React, { useEffect, useState } from 'react';
import { useNavigate, useLocation } from 'react-router-dom';
import { MessageCircle, Plus } from 'lucide-react';
import { get_my_servers, type JSServerSummary } from '../../../wasm/pkg';
import { getAvatarColor, getInitials } from '@/utils/avatarGenerator';
import CreateServerModal from '@/components/modals/CreateServerModal';
import { useUnread } from '@/context/UnreadProvider';

function ServerSidebar(): React.JSX.Element {
    const navigate = useNavigate();
    const location = useLocation();
    const [servers, setServers] = useState<JSServerSummary[]>([]);
    const [showCreateModal, setShowCreateModal] = useState(false);
    const { hasAnyDmUnread, hasServerUnread } = useUnread();

    const loadServers = async () => {
        const s = await get_my_servers();
        setServers(s);
    };

    useEffect(() => {
        loadServers();
        const interval = setInterval(loadServers, 10_000);
        return () => clearInterval(interval);
    }, [location.pathname]);

    const activeServerId = location.pathname.startsWith('/server/')
        ? parseInt(location.pathname.split('/')[2])
        : null;

    return (
        <>
            <div className="w-[72px] bg-dc-bg-tertiary flex flex-col items-center py-3 gap-2 overflow-y-auto flex-shrink-0">
                {/* DMs */}
                <div className="relative">
                    <button
                        onClick={() => navigate('/dms')}
                        className={`w-12 h-12 rounded-[24px] hover:rounded-[16px] transition-all duration-200 flex items-center justify-center ${
                            location.pathname.startsWith('/dms') ? 'bg-dc-blurple rounded-[16px]' : 'bg-dc-bg-primary hover:bg-dc-blurple'
                        }`}
                    >
                        <MessageCircle size={24} className="text-white" />
                    </button>
                    {hasAnyDmUnread() && (
                        <span className="absolute -bottom-0.5 -right-0.5 w-4 h-4 bg-red-500 rounded-full border-2 border-dc-bg-tertiary" />
                    )}
                </div>

                <div className="w-8 h-[2px] bg-dc-border rounded-full mx-auto" />

                {/* Joined Servers */}
                {servers.map(server => (
                    <div key={server.id} className="relative">
                        <button
                            onClick={() => navigate(`/server/${server.id}`)}
                            title={server.name}
                            className={`w-12 h-12 rounded-[24px] hover:rounded-[16px] transition-all duration-200 flex items-center justify-center text-white font-bold text-sm ${
                                activeServerId === Number(server.id)
                                    ? 'rounded-[16px]'
                                    : ''
                            }`}
                            style={{ backgroundColor: getAvatarColor(server.name) }}
                        >
                            {getInitials(server.name)}
                        </button>
                        {hasServerUnread(Number(server.id)) && (
                            <span className="absolute -bottom-0.5 -right-0.5 w-4 h-4 bg-red-500 rounded-full border-2 border-dc-bg-tertiary" />
                        )}
                    </div>
                ))}

                {/* Create Server */}
                <button
                    onClick={() => setShowCreateModal(true)}
                    className="w-12 h-12 rounded-[24px] hover:rounded-[16px] transition-all duration-200 flex items-center justify-center bg-dc-bg-primary hover:bg-dc-green text-dc-green hover:text-white"
                >
                    <Plus size={24} />
                </button>
            </div>

            <CreateServerModal
                isOpen={showCreateModal}
                onClose={() => setShowCreateModal(false)}
                onCreated={() => { setShowCreateModal(false); loadServers(); }}
            />
        </>
    );
}

export default ServerSidebar;
