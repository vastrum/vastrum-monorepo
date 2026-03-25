import React, { useEffect, useState } from 'react';
import { get_my_dms, type JSDmSummary } from '../../wasm/pkg';
import DmSidebar from '@/components/layout/DmSidebar';
import NewDmModal from '@/components/modals/NewDmModal';
import { resolveDisplayNames } from '@/utils/resolveDisplayNames';
import { useMobileSidebar } from '@/context/MobileSidebarProvider';
import { MessageCircle, Menu } from 'lucide-react';

function DmList(): React.JSX.Element {
    const [dms, setDms] = useState<JSDmSummary[]>([]);
    const [names, setNames] = useState<Record<string, string>>({});
    const [showNewDm, setShowNewDm] = useState(false);
    const [loading, setLoading] = useState(true);
    const { sidebarOpen, openSidebar, closeSidebar } = useMobileSidebar();

    const loadDms = async () => {
        setLoading(true);
        const myDms = await get_my_dms(BigInt(200), BigInt(0));
        setDms(myDms);
        setNames(await resolveDisplayNames(myDms));
        setLoading(false);
    };

    useEffect(() => { loadDms(); }, []);

    // Poll DM list (~10s)
    useEffect(() => {
        const poll = async () => {
            const myDms = await get_my_dms(BigInt(200), BigInt(0));
            setDms(myDms);
        };
        const interval = setInterval(poll, 10_000);
        return () => clearInterval(interval);
    }, []);

    return (
        <>
            <div className={`fixed inset-y-0 left-0 z-50 flex transition-transform duration-200 ${sidebarOpen ? 'translate-x-[72px]' : '-translate-x-full'} md:relative md:translate-x-0 md:z-auto md:transition-none`}>
                <DmSidebar
                    dms={dms}
                    names={names}
                    onDmClick={() => {}}
                    onNewMessage={() => setShowNewDm(true)}
                    onNavigate={closeSidebar}
                />
            </div>

            {/* Main area */}
            <div className="flex-1 flex flex-col bg-dc-bg-primary">
                <div className="h-12 flex items-center px-4 border-b border-dc-bg-tertiary shadow-sm flex-shrink-0 md:hidden">
                    <button onClick={openSidebar} className="mr-2 text-dc-text-muted hover:text-dc-text">
                        <Menu size={20} />
                    </button>
                    <span className="font-semibold text-white">Direct Messages</span>
                </div>
                <div className="flex-1 flex items-center justify-center">
                    {loading ? (
                        <p className="text-dc-text-muted">Loading...</p>
                    ) : dms.length === 0 ? (
                        <div className="text-center">
                            <MessageCircle size={48} className="text-dc-text-muted mx-auto mb-3 opacity-50" />
                            <p className="text-dc-text-muted mb-1">No dms yet</p>
                            <button
                                onClick={() => setShowNewDm(true)}
                                className="text-dc-blurple hover:underline text-sm"
                            >
                                Send dm
                            </button>
                        </div>
                    ) : (
                        <p className="text-dc-text-muted">Select a conversation</p>
                    )}
                </div>
            </div>

            <NewDmModal
                isOpen={showNewDm}
                onClose={() => setShowNewDm(false)}
                onSent={() => { setShowNewDm(false); loadDms(); }}
            />
        </>
    );
}

export default DmList;
