import type { Token, Trade } from '../types';

interface RecentTradesProps {
    trades: Trade[];
    token0: Token;
    token1: Token;
    token0Decimals: number;
    token1Decimals: number;
    isLoading: boolean;
}

function formatAmount(amount: bigint, decimals: number): string {
    const divisor = 10n ** BigInt(decimals);
    const whole = amount / divisor;
    const remainder = amount % divisor;
    const remainderStr = remainder.toString().padStart(decimals, '0').slice(0, 4);
    return `${whole}.${remainderStr}`;
}

function shortenTxHash(hash: string): string {
    if (!hash) return '';
    return `${hash.slice(0, 6)}...${hash.slice(-4)}`;
}

export function RecentTrades({
    trades,
    token0,
    token1,
    token0Decimals,
    token1Decimals,
    isLoading,
}: RecentTradesProps) {
    if (isLoading && trades.length === 0) {
        return (
            <div className="bg-app-bg-secondary border border-app-border rounded-3xl p-6">
                <h3 className="text-app-text-primary text-lg font-semibold mb-4">Recent Trades</h3>
                <div className="text-app-text-secondary text-center py-8">
                    Loading trades...
                </div>
            </div>
        );
    }

    if (trades.length === 0) {
        return (
            <div className="bg-app-bg-secondary border border-app-border rounded-3xl p-6">
                <h3 className="text-app-text-primary text-lg font-semibold mb-4">Recent Trades</h3>
                <div className="text-app-text-secondary text-center py-8">
                    No recent trades found
                </div>
            </div>
        );
    }

    return (
        <div className="bg-app-bg-secondary border border-app-border rounded-3xl p-6">
            <h3 className="text-app-text-primary text-lg font-semibold mb-4">Recent Trades</h3>
            {/* Mobile card layout */}
            <div className="md:hidden flex flex-col gap-3">
                {trades.map((trade, index) => {
                    const isBuy = trade.amount0In > 0n;
                    const token0Amount = isBuy ? trade.amount0In : trade.amount0Out;
                    const token1Amount = isBuy ? trade.amount1Out : trade.amount1In;

                    return (
                        <div
                            key={`mobile-${trade.txHash}-${index}`}
                            className="bg-app-bg-tertiary rounded-xl p-3 border border-app-border/50"
                        >
                            <div className="flex justify-between items-center mb-2">
                                <span className={`text-sm font-medium ${isBuy ? 'text-app-accent-green' : 'text-app-accent-red'}`}>
                                    {isBuy ? 'Buy' : 'Sell'}
                                </span>
                                <a
                                    href={`https://etherscan.io/tx/${trade.txHash}`}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="text-app-accent-blue text-xs hover:underline"
                                >
                                    {shortenTxHash(trade.txHash)}
                                </a>
                            </div>
                            <div className="flex justify-between items-center text-sm">
                                <div>
                                    <span className="text-app-text-secondary text-xs">{token0.symbol}</span>
                                    <p className="text-app-text-primary">{formatAmount(token0Amount, token0Decimals)}</p>
                                </div>
                                <div className="text-right">
                                    <span className="text-app-text-secondary text-xs">{token1.symbol}</span>
                                    <p className="text-app-text-primary">{formatAmount(token1Amount, token1Decimals)}</p>
                                </div>
                            </div>
                            <div className="text-app-text-secondary text-xs mt-2">
                                Block {trade.blockNumber.toString()}
                            </div>
                        </div>
                    );
                })}
            </div>

            {/* Desktop table layout */}
            <div className="hidden md:block overflow-x-auto">
                <table className="w-full">
                    <thead>
                        <tr className="text-app-text-secondary text-sm border-b border-app-border">
                            <th className="text-left pb-3 font-medium">Type</th>
                            <th className="text-right pb-3 font-medium">{token0.symbol}</th>
                            <th className="text-right pb-3 font-medium">{token1.symbol}</th>
                            <th className="text-right pb-3 font-medium">Block</th>
                            <th className="text-right pb-3 font-medium">Tx</th>
                        </tr>
                    </thead>
                    <tbody>
                        {trades.map((trade, index) => {
                            const isBuy = trade.amount0In > 0n;
                            const token0Amount = isBuy ? trade.amount0In : trade.amount0Out;
                            const token1Amount = isBuy ? trade.amount1Out : trade.amount1In;

                            return (
                                <tr
                                    key={`${trade.txHash}-${index}`}
                                    className="border-b border-app-border/50 last:border-0"
                                >
                                    <td className="py-3">
                                        <span className={`text-sm font-medium ${isBuy ? 'text-app-accent-green' : 'text-app-accent-red'}`}>
                                            {isBuy ? 'Buy' : 'Sell'}
                                        </span>
                                    </td>
                                    <td className="text-right py-3">
                                        <span className="text-app-text-primary text-sm">
                                            {formatAmount(token0Amount, token0Decimals)}
                                        </span>
                                    </td>
                                    <td className="text-right py-3">
                                        <span className="text-app-text-primary text-sm">
                                            {formatAmount(token1Amount, token1Decimals)}
                                        </span>
                                    </td>
                                    <td className="text-right py-3">
                                        <span className="text-app-text-secondary text-sm">
                                            {trade.blockNumber.toString()}
                                        </span>
                                    </td>
                                    <td className="text-right py-3">
                                        <a
                                            href={`https://etherscan.io/tx/${trade.txHash}`}
                                            target="_blank"
                                            rel="noopener noreferrer"
                                            className="text-app-accent-blue text-sm hover:underline"
                                        >
                                            {shortenTxHash(trade.txHash)}
                                        </a>
                                    </td>
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>
        </div>
    );
}
