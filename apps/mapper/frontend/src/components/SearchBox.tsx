import { useState } from 'react';
import { Place, RANKS } from '../search/useSearch';

interface SearchBoxProps {
    search: string;
    results: Place[];
    onSearchChange: (value: string) => void;
    onSearchKey: (e: React.KeyboardEvent<HTMLInputElement>) => void;
    onSelectPlace: (place: Place) => void;
    onClearResults: () => void;
}

export default function SearchBox({
    search,
    results,
    onSearchChange,
    onSearchKey,
    onSelectPlace,
    onClearResults,
}: SearchBoxProps) {
    const [searchFocused, setSearchFocused] = useState(false);

    return (
        <div className="absolute top-3 left-3 right-3 sm:right-auto sm:w-[376px] z-30"
             style={{ pointerEvents: 'none' }}>
            <div className="search-pill" data-focused={searchFocused}
                 style={{ pointerEvents: 'auto' }}>
                <svg width="15" height="15" viewBox="0 0 24 24" fill="none"
                     stroke="#94a3b8" strokeWidth="2" strokeLinecap="round">
                    <circle cx="11" cy="11" r="8" />
                    <line x1="21" y1="21" x2="16.65" y2="16.65" />
                </svg>
                <input
                    type="text"
                    value={search}
                    onChange={e => onSearchChange(e.target.value)}
                    onFocus={() => setSearchFocused(true)}
                    onBlur={() => { setSearchFocused(false); setTimeout(() => onClearResults(), 150); }}
                    onKeyDown={onSearchKey}
                    placeholder="Search places or coordinates..."
                    className="search-input"
                />
            </div>
            {results.length > 0 && (
                <div className="search-results" style={{ pointerEvents: 'auto' }}>
                    {results.map((place, i) => (
                        <div
                            key={`${place.name}-${place.lat}-${i}`}
                            className="search-result-row"
                            onMouseDown={() => onSelectPlace(place)}
                        >
                            <span className="search-result-name">{place.name}</span>
                            <span className="search-result-kind">
                                {RANKS[Math.min(place.rank, RANKS.length - 1)].label}
                            </span>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}
