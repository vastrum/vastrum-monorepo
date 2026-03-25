import { truncateAddress } from '../../utils/avatarGenerator';
import { stringToColor } from '../../utils/colorUtils';

export function extractQuote(content: string): { author: string; quoted: string; rest: string } | null {
    if (!content.startsWith('<!--quote:')) return null;
    const end = content.indexOf('-->');
    if (end === -1) return null;
    const meta = content.slice(10, end);
    const sep = meta.indexOf(':');
    if (sep === -1) return null;
    return {
        author: meta.slice(0, sep),
        quoted: meta.slice(sep + 1),
        rest: content.slice(end + 3).trimStart(),
    };
}

export default function QuoteCard({ author, quoted, onDismiss }: {
    author: string;
    quoted: string;
    onDismiss?: () => void;
}) {
    const shortAddr = truncateAddress(author);
    return (
        <div className="quote-card">
            <div
                className="quote-card-avatar"
                style={{ backgroundColor: stringToColor(author) }}
            >
                {author.slice(0, 2).toUpperCase()}
            </div>
            <div className="quote-card-body">
                <div className="quote-card-author">{shortAddr}</div>
                <div className="quote-card-content">{quoted}</div>
            </div>
            {onDismiss && (
                <button className="quote-card-dismiss" onClick={onDismiss}>
                    &times;
                </button>
            )}
        </div>
    );
}
