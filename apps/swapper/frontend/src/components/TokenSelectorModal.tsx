import { useState, useMemo } from 'react';
import { isAddress } from 'viem';
import { CloseIcon, SearchIcon } from './Icons';
import { TOKEN_LIST, NATIVE_ETH_ADDRESS } from '../constants';
import type { Token } from '../types';
import type { Address } from 'viem';

interface TokenSelectorModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSelect: (token: Token) => void;
    selectedToken: Token;
    otherToken: Token;
}

function createCustomToken(address: Address): Token {
    return {
        address,
        symbol: `${address.slice(0, 6)}...${address.slice(-4)}`,
        name: 'Custom Token',
        decimals: 18,
        logo: '',
    };
}

export function TokenSelectorModal({
    isOpen,
    onClose,
    onSelect,
    selectedToken,
    otherToken,
}: TokenSelectorModalProps) {
    const [searchQuery, setSearchQuery] = useState('');

    const filteredTokens = useMemo(() => {
        if (!searchQuery) return TOKEN_LIST;
        const query = searchQuery.toLowerCase();
        return TOKEN_LIST.filter(
            (token) =>
                token.symbol.toLowerCase().includes(query) ||
                token.name.toLowerCase().includes(query) ||
                (token.address !== NATIVE_ETH_ADDRESS && token.address.toLowerCase().includes(query))
        );
    }, [searchQuery]);

    const customToken = useMemo(() => {
        const trimmed = searchQuery.trim();
        if (!isAddress(trimmed)) return null;

        const existsInList = TOKEN_LIST.some(
            (token) => token.address.toLowerCase() === trimmed.toLowerCase()
        );
        if (existsInList) return null;

        return createCustomToken(trimmed as Address);
    }, [searchQuery]);

    const handleSelect = (token: Token) => {
        onSelect(token);
        onClose();
        setSearchQuery('');
    };

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
            <div
                className="absolute inset-0 bg-black/60 backdrop-blur-sm"
                onClick={onClose}
            />

            <div className="relative bg-app-bg-secondary border border-app-border rounded-3xl w-full max-w-[420px] max-h-[80vh] flex flex-col mx-4">
                <div className="flex justify-between items-center p-4 border-b border-app-border">
                    <h2 className="text-app-text-primary text-lg font-semibold">Select a token</h2>
                    <button
                        onClick={onClose}
                        className="text-app-text-secondary hover:text-app-text-primary transition-colors p-1"
                    >
                        <CloseIcon />
                    </button>
                </div>

                <div className="p-4">
                    <div className="relative">
                        <div className="absolute left-3 top-1/2 -translate-y-1/2 text-app-text-secondary">
                            <SearchIcon />
                        </div>
                        <input
                            type="text"
                            placeholder="Search name or paste address"
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            className="w-full bg-app-bg-tertiary border border-app-border rounded-xl py-3 pl-10 pr-4 text-app-text-primary placeholder-app-text-secondary outline-none focus:border-app-accent-blue transition-colors"
                        />
                    </div>
                </div>

                <div className="px-4 pb-2">
                    <div className="flex flex-wrap gap-2">
                        {TOKEN_LIST.slice(0, 6).map((token) => {
                            const isSelected = token.symbol === selectedToken.symbol;
                            const isOther = token.symbol === otherToken.symbol;
                            return (
                                <button
                                    key={token.symbol}
                                    onClick={() => !isOther && handleSelect(token)}
                                    disabled={isOther}
                                    className={`flex items-center gap-2 px-3 py-2 rounded-2xl border transition-colors ${isSelected
                                        ? 'bg-app-accent-blue/20 border-app-accent-blue'
                                        : isOther
                                            ? 'bg-app-bg-tertiary border-app-border opacity-50 cursor-not-allowed'
                                            : 'bg-app-bg-tertiary border-app-border hover:border-app-text-secondary'
                                        }`}
                                >
                                    <img
                                        src={token.logo}
                                        alt={token.symbol}
                                        className="w-5 h-5 rounded-full"
                                    />
                                    <span className="text-app-text-primary text-sm font-medium">
                                        {token.symbol}
                                    </span>
                                </button>
                            );
                        })}
                    </div>
                </div>

                <div className="border-t border-app-border mx-4" />

                <div className="flex-1 overflow-y-auto p-2 scrollbar-thin">
                    {customToken && (
                        (() => {
                            const isOther = otherToken.address.toLowerCase() === customToken.address.toLowerCase();
                            return (
                                <button
                                    onClick={() => !isOther && handleSelect(customToken)}
                                    disabled={isOther}
                                    className={`w-full flex items-center gap-3 p-3 rounded-xl transition-colors mb-2 border border-app-accent-blue/50 ${isOther
                                            ? 'opacity-50 cursor-not-allowed'
                                            : 'hover:bg-app-accent-blue/10'
                                        }`}
                                >
                                    <div className="flex-1 text-left">
                                        <div className="text-app-text-primary font-medium">
                                            {customToken.symbol}
                                        </div>
                                        <div className="text-app-text-secondary text-sm">
                                        </div>
                                    </div>
                                </button>
                            );
                        })()
                    )}
                    {filteredTokens.length === 0 && !customToken ? (
                        <div className="text-center text-app-text-secondary py-8">
                            No tokens found
                        </div>
                    ) : (
                        filteredTokens.map((token) => {
                            const isSelected = token.symbol === selectedToken.symbol;
                            const isOther = token.symbol === otherToken.symbol;
                            return (
                                <button
                                    key={token.address}
                                    onClick={() => !isOther && handleSelect(token)}
                                    disabled={isOther}
                                    className={`w-full flex items-center gap-3 p-3 rounded-xl transition-colors ${isOther
                                        ? 'opacity-50 cursor-not-allowed'
                                        : isSelected
                                            ? 'bg-app-accent-blue/10'
                                            : 'hover:bg-app-bg-tertiary'
                                        }`}
                                >
                                    <img
                                        src={token.logo}
                                        alt={token.symbol}
                                        className="w-8 h-8 rounded-full"
                                    />
                                    <div className="flex-1 text-left">
                                        <div className="text-app-text-primary font-medium">
                                            {token.symbol}
                                        </div>
                                        <div className="text-app-text-secondary text-sm">
                                            {token.name}
                                        </div>
                                    </div>
                                    {isSelected && (
                                        <div className="w-2 h-2 rounded-full bg-app-accent-blue" />
                                    )}
                                </button>
                            );
                        })
                    )}
                </div>
            </div>
        </div>
    );
}
