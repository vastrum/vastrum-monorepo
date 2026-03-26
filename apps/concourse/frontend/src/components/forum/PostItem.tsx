import { truncateAddress } from '../../utils/avatarGenerator';
import { stringToColor } from '../../utils/colorUtils';
import { formatRelativeTime } from '../../utils/timeUtils';
import { extractQuote } from './QuoteCard';
import QuoteCard from './QuoteCard';
import MarkdownRenderer from '../common/MarkdownRenderer';

export default function PostItem({ author, content, timestamp, index, onQuote, onDelete, deleteConfirming }: {
    author: string;
    content: string;
    timestamp: number;
    index: number;
    onQuote: () => void;
    onDelete?: () => void;
    deleteConfirming?: boolean;
}) {
    return (
        <article style={{
            display: 'flex',
            gap: 16,
            padding: '20px 0',
            borderBottom: '1px solid #e9e9e9',
        }}>
            {/* Avatar */}
            <div style={{
                width: 45,
                height: 45,
                borderRadius: '50%',
                backgroundColor: stringToColor(author),
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                flexShrink: 0,
            }}>
                <span style={{
                    fontSize: 14,
                    fontWeight: 600,
                    color: '#fff',
                }}>
                    {author.slice(0, 2).toUpperCase()}
                </span>
            </div>

            {/* Content */}
            <div style={{ flex: 1, minWidth: 0 }}>
                {/* Header */}
                <div style={{
                    display: 'flex',
                    alignItems: 'center',
                    gap: 8,
                    marginBottom: 8,
                }}>
                    <span style={{
                        fontWeight: 500,
                        color: '#08c',
                        fontSize: 14,
                    }}>
                        {truncateAddress(author)}
                    </span>
                    <span style={{
                        color: '#919191',
                        fontSize: 13,
                    }}>
                        {formatRelativeTime(timestamp)}
                    </span>
                    <span style={{
                        marginLeft: 'auto',
                        display: 'flex',
                        alignItems: 'center',
                        gap: 8,
                    }}>
                        <button
                            className="quote-btn"
                            onClick={onQuote}
                            style={{
                                background: 'none',
                                border: '1px solid #ddd',
                                borderRadius: 3,
                                padding: '2px 8px',
                                fontSize: 11,
                                color: '#999',
                                cursor: 'pointer',
                            }}
                        >
                            Quote
                        </button>
                        {onDelete && (
                            <button
                                className="quote-btn delete-btn"
                                onClick={onDelete}
                                style={{
                                    background: deleteConfirming ? '#fee' : 'none',
                                    border: '1px solid #ddd',
                                    borderRadius: 3,
                                    padding: '2px 8px',
                                    fontSize: 11,
                                    color: '#999',
                                    cursor: 'pointer',
                                }}
                            >
                                {deleteConfirming ? 'Confirm?' : 'Delete'}
                            </button>
                        )}
                        <span style={{
                            color: '#ccc',
                            fontSize: 12,
                        }}>
                            #{index + 1}
                        </span>
                    </span>
                </div>

                {/* Body */}
                {(() => {
                    const quote = extractQuote(content);
                    if (quote) {
                        return (
                            <>
                                <QuoteCard author={quote.author} quoted={quote.quoted} />
                                <MarkdownRenderer content={quote.rest} />
                            </>
                        );
                    }
                    return <MarkdownRenderer content={content} />;
                })()}
            </div>
        </article>
    );
}
