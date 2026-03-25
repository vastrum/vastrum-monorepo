import React, { useState, useEffect } from 'react';
import { Settings } from 'lucide-react';
import Avatar from '@/components/common/Avatar';
import { truncateAddress } from '@/utils/avatarGenerator';
import UserSettingsModal from '@/components/modals/UserSettingsModal';
import { get_my_pubkey, get_user_profile } from '../../../wasm/pkg';

function UserPanel(): React.JSX.Element {
    const [pubkey, setPubkey] = useState('');
    const [displayName, setDisplayName] = useState('');
    const [showSettings, setShowSettings] = useState(false);

    const fetchIdentity = async () => {
        try {
            const pk = await get_my_pubkey();
            setPubkey(pk);
            const profile = await get_user_profile(pk);
            if (profile.display_name) {
                setDisplayName(profile.display_name);
            }
        } catch (e) {
            console.error('Failed to fetch identity:', e);
        }
    };

    useEffect(() => {
        fetchIdentity();
    }, []);

    const handleSaved = (newName: string) => {
        setDisplayName(newName);
        setShowSettings(false);
    };

    return (
        <>
            <div className="h-[52px] bg-dc-bg-tertiary border-t border-dc-bg-tertiary flex items-center px-2 gap-2">
                <Avatar identifier={pubkey} name={displayName || pubkey} size={32} />
                <div className="flex-1 min-w-0">
                    <div className="text-sm font-medium text-white truncate">
                        {displayName || truncateAddress(pubkey)}
                    </div>
                </div>
                <button
                    onClick={() => setShowSettings(true)}
                    className="text-dc-text-muted hover:text-dc-text p-1 rounded transition-colors"
                >
                    <Settings size={18} />
                </button>
            </div>

            <UserSettingsModal
                isOpen={showSettings}
                onClose={() => setShowSettings(false)}
                pubkey={pubkey}
                currentDisplayName={displayName}
                onSaved={handleSaved}
            />
        </>
    );
}

export default UserPanel;
