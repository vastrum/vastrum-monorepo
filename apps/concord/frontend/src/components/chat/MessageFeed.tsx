import React, { useEffect, useRef } from 'react';
import MessageItem from './MessageItem';

interface Message {
    id: number;
    content: string;
    author: string;
    timestamp: number;
}

interface MemberLookup {
    [pubkey: string]: string;
}

interface MessageFeedProps {
    messages: Message[];
    memberNames: MemberLookup;
    myPubkey: string;
    ownerPubkey: string;
    onDeleteMessage: (messageId: number) => void;
}

function MessageFeed({ messages, memberNames, myPubkey, ownerPubkey, onDeleteMessage }: MessageFeedProps): React.JSX.Element {
    const bottomRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
    }, [messages.length]);

    const canDeleteMsg = (msg: Message): boolean => {
        if (msg.author === myPubkey) return true;
        if (myPubkey === ownerPubkey) return true;
        return false;
    };

    const shouldShowHeader = (msg: Message, idx: number): boolean => {
        if (idx === 0) return true;
        const prev = messages[idx - 1];
        if (prev.author !== msg.author) return true;
        // Show header if more than 5 minutes apart
        if (Number(msg.timestamp) - Number(prev.timestamp) > 300) return true;
        return false;
    };

    if (messages.length === 0) {
        return (
            <div className="flex-1 flex items-center justify-center text-dc-text-muted">
                <p>No messages yet</p>
            </div>
        );
    }

    return (
        <div className="flex-1 overflow-y-auto">
            <div className="min-h-full flex flex-col justify-end">
                {messages.map((msg, idx) => (
                    <MessageItem
                        key={String(msg.id)}
                        id={msg.id}
                        content={msg.content}
                        author={msg.author}
                        timestamp={msg.timestamp}
                        authorName={memberNames[msg.author]}
                        myPubkey={myPubkey}
                        canDelete={canDeleteMsg(msg)}
                        onDelete={() => onDeleteMessage(msg.id)}
                        showHeader={shouldShowHeader(msg, idx)}
                    />
                ))}
                <div ref={bottomRef} />
            </div>
        </div>
    );
}

export default MessageFeed;
