import React from 'react';
import MarkdownRenderer from './MarkdownRenderer';
import { truncateAddress } from '../../utils/avatarGenerator';

interface CommentBoxProps {
    author: string;
    content: string;
    timeAgo: string;
    badge?: string;
}

function CommentBox({ author, content, timeAgo, badge }: CommentBoxProps): React.JSX.Element {
    return (
        <div className="comment-box">
            <div className="bg-app-bg-tertiary px-4 py-3 border-b border-app-border flex items-center justify-between min-w-0">
                <div className="flex items-center gap-2 text-sm min-w-0">
                    <strong className="text-app-text-primary font-semibold truncate">{truncateAddress(author)}</strong>
                    {badge && (
                        <span className="bg-app-border px-2 py-0.5 rounded text-xs text-app-text-secondary">{badge}</span>
                    )}
                    <span className="text-app-text-secondary">{timeAgo}</span>
                </div>
            </div>
            <div className="p-4">
                <MarkdownRenderer content={content} />
            </div>
        </div>
    );
}

export default CommentBox;
