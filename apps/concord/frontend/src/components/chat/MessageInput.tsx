import React, { useState } from 'react';
import { SendHorizontal } from 'lucide-react';

interface MessageInputProps {
    placeholder: string;
    onSend: (content: string) => void;
    disabled?: boolean;
}

function MessageInput({ placeholder, onSend, disabled }: MessageInputProps): React.JSX.Element {
    const [content, setContent] = useState('');

    const handleSend = () => {
        const trimmed = content.trim();
        if (!trimmed || disabled) return;
        onSend(trimmed);
        setContent('');
    };

    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            handleSend();
        }
    };

    return (
        <div className="px-4 pb-6 pt-1">
            <div className="flex items-center bg-dc-input rounded-lg px-4">
                <input
                    type="text"
                    value={content}
                    onChange={e => setContent(e.target.value)}
                    onKeyDown={handleKeyDown}
                    placeholder={placeholder}
                    disabled={disabled}
                    className="flex-1 bg-transparent py-2.5 text-sm text-dc-text placeholder-dc-text-muted outline-none"
                />
                <button
                    onClick={handleSend}
                    disabled={disabled || !content.trim()}
                    className="text-dc-text-muted hover:text-dc-text disabled:opacity-30 transition-colors p-1"
                >
                    <SendHorizontal size={20} />
                </button>
            </div>
        </div>
    );
}

export default MessageInput;
