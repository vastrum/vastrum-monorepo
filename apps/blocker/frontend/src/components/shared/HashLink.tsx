import { Link } from 'react-router-dom';
import { truncateHash } from '../../utils/format';

interface HashLinkProps {
    hash: string;
    to: string;
    truncate?: number;
}

function HashLink({ hash, to, truncate = 8 }: HashLinkProps) {
    return (
        <Link
            to={to}
            className="font-mono text-blocker-accent hover:text-blocker-accent-hover transition-colors"
            title={hash}
        >
            {truncateHash(hash, truncate)}
        </Link>
    );
}

export default HashLink;
