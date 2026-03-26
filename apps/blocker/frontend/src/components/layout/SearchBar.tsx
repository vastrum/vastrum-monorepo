import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Search } from 'lucide-react';

function SearchBar() {
    const [query, setQuery] = useState('');
    const navigate = useNavigate();

    const handleSearch = (e: React.FormEvent) => {
        e.preventDefault();
        const q = query.trim();
        if (!q) return;

        // Number = block height
        if (/^\d+$/.test(q)) {
            navigate(`/block/${q}`);
        }
        // 64 char hex = tx hash or account pubkey (try tx first via UI)
        else if (/^[0-9a-f]{64}$/i.test(q)) {
            navigate(`/tx/${q}`);
        }
        // base32 = site_id or block hash
        else if (/^[a-z2-7]{52}$/i.test(q)) {
            navigate(`/site/${q}`);
        }

        setQuery('');
    };

    return (
        <form onSubmit={handleSearch} className="mb-6">
            <div className="relative">
                <Search size={16} className="absolute left-3 top-1/2 -translate-y-1/2 text-blocker-text-muted" />
                <input
                    type="text"
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    placeholder="Search by block height, tx hash, account, or site ID..."
                    className="w-full bg-blocker-surface border border-blocker-border rounded-lg pl-10 pr-4 py-2.5 text-sm text-blocker-text-primary placeholder-blocker-text-muted focus:outline-none focus:border-blocker-accent transition-colors"
                />
            </div>
        </form>
    );
}

export default SearchBar;
