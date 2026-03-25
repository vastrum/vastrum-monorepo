import { ChevronDownIcon } from './Icons';
import type { Token } from '../types';

interface TokenInputProps {
    label: string;
    token: Token;
    amount: string;
    onAmountChange?: (value: string) => void;
    onTokenClick: () => void;
    readOnly?: boolean;
}

function getFontSize(amount: string): string {
    const len = amount.length;
    if (len <= 10) return 'text-3xl';
    if (len <= 16) return 'text-2xl';
    if (len <= 22) return 'text-xl';
    return 'text-lg';
}

export function TokenInput({
    label,
    token,
    amount,
    onAmountChange,
    onTokenClick,
    readOnly = false,
}: TokenInputProps) {
    return (
        <div className="bg-app-bg-tertiary rounded-2xl p-4">
            <div className="flex justify-between items-center mb-2">
                <span className="text-app-text-secondary text-sm">{label}</span>
            </div>
            <div className="flex justify-between items-center gap-4">
                <input
                    type="text"
                    inputMode="decimal"
                    value={amount}
                    onChange={(e) => onAmountChange?.(e.target.value)}
                    placeholder="0"
                    readOnly={readOnly}
                    className={`bg-transparent ${getFontSize(amount)} font-medium text-app-text-primary outline-none flex-1 min-w-0 overflow-hidden text-ellipsis ${readOnly ? 'cursor-default' : ''
                        }`}
                />
                <button
                    onClick={onTokenClick}
                    className="flex items-center gap-2 bg-app-bg-secondary hover:bg-app-hover px-3 py-2 rounded-2xl transition-colors"
                >
                    {token.logo && (
                        <img
                            src={token.logo}
                            alt={token.symbol}
                            className="w-6 h-6 rounded-full"
                        />
                    )}
                    <span className="text-app-text-primary font-semibold">{token.symbol}</span>
                    <ChevronDownIcon />
                </button>
            </div>
        </div>
    );
}
