import { formatUnits } from 'viem';
import { WETH_ADDRESS, NATIVE_ETH_ADDRESS } from '../constants';
import type { PairInfo, Token } from '../types';

interface PoolSidebarProps {
    pairInfo: PairInfo | null;
    inputToken: Token;
    outputToken: Token;
    isLoading: boolean;
    blockNumber: bigint | null;
}

export function PoolSidebar({ pairInfo, inputToken, outputToken, isLoading, blockNumber }: PoolSidebarProps) {
    if (isLoading) {
        return (
            <div className="bg-app-bg-secondary border border-app-border rounded-2xl p-4 w-full max-w-[320px]">
                <h3 className="text-app-text-primary font-semibold mb-4">Pool Info</h3>
                <div className="animate-pulse space-y-3">
                    <div className="h-4 bg-app-bg-tertiary rounded w-3/4"></div>
                    <div className="h-4 bg-app-bg-tertiary rounded w-1/2"></div>
                    <div className="h-4 bg-app-bg-tertiary rounded w-2/3"></div>
                </div>
            </div>
        );
    }

    if (!pairInfo) {
        return (
            <div className="bg-app-bg-secondary border border-app-border rounded-2xl p-4 w-full max-w-[320px]">
                <h3 className="text-app-text-primary font-semibold mb-4">Pool Info</h3>
                <p className="text-app-text-secondary text-sm">No liquidity pool found for this pair.</p>
            </div>
        );
    }

    const inputAddress = inputToken.address === NATIVE_ETH_ADDRESS ? WETH_ADDRESS : inputToken.address;
    const outputAddress = outputToken.address === NATIVE_ETH_ADDRESS ? WETH_ADDRESS : outputToken.address;
    const inputIsToken0 = inputAddress.toLowerCase() === pairInfo.token0.toLowerCase();
    const outputIsToken0 = outputAddress.toLowerCase() === pairInfo.token0.toLowerCase();

    const inputReserve = inputIsToken0 ? pairInfo.reserve0 : pairInfo.reserve1;
    const outputReserve = outputIsToken0 ? pairInfo.reserve0 : pairInfo.reserve1;

    const inputReserveFormatted = Number(formatUnits(inputReserve, inputToken.decimals));
    const outputReserveFormatted = Number(formatUnits(outputReserve, outputToken.decimals));

    const rate = outputReserveFormatted / inputReserveFormatted;

    return (
        <div className="bg-app-bg-secondary border border-app-border rounded-2xl p-4 w-full max-w-[320px]">
            <h3 className="text-app-text-primary font-semibold mb-4">Pool Info</h3>

            <div className="space-y-4">
                <div>
                    <div className="text-app-text-secondary text-xs mb-1">Pool Address</div>
                    <span className="text-app-text-primary font-mono text-xs break-all">
                        {pairInfo.pairAddress}
                    </span>
                </div>

                <div>
                    <div className="text-app-text-secondary text-xs mb-1">Trading Pair</div>
                    <div className="flex items-center gap-2">
                        <span className="text-app-text-primary font-medium">{inputToken.symbol}</span>
                        <span className="text-app-text-secondary">/</span>
                        <span className="text-app-text-primary font-medium">{outputToken.symbol}</span>
                    </div>
                </div>

                <div className="bg-app-bg-tertiary rounded-xl p-3 space-y-2">
                    <div className="text-app-text-secondary text-xs mb-2">Reserves</div>
                    <div className="flex justify-between items-center">
                        <span className="text-app-text-secondary text-sm">{inputToken.symbol}</span>
                        <span className="text-app-text-primary text-sm font-mono">
                            {inputReserveFormatted.toLocaleString(undefined, { maximumFractionDigits: 4 })}
                        </span>
                    </div>
                    <div className="flex justify-between items-center">
                        <span className="text-app-text-secondary text-sm">{outputToken.symbol}</span>
                        <span className="text-app-text-primary text-sm font-mono">
                            {outputReserveFormatted.toLocaleString(undefined, { maximumFractionDigits: 4 })}
                        </span>
                    </div>
                </div>

                <div>
                    <div className="text-app-text-secondary text-xs mb-1">Spot Price</div>
                    <div className="text-app-text-primary text-sm">
                        1 {inputToken.symbol} = {rate.toLocaleString(undefined, { maximumFractionDigits: 6 })} {outputToken.symbol}
                    </div>
                    <div className="text-app-text-secondary text-xs">
                        1 {outputToken.symbol} = {(1 / rate).toLocaleString(undefined, { maximumFractionDigits: 6 })} {inputToken.symbol}
                    </div>
                </div>

                <div>
                    <div className="text-app-text-secondary text-xs mb-1">Last Updated</div>
                    <div className="text-app-text-primary text-sm font-mono">
                        Block #{blockNumber !== null ? String(blockNumber) : '...'}
                    </div>
                </div>

                <div className="pt-3 border-t border-app-border">
                    <div className="flex justify-between text-xs">
                        <span className="text-app-text-secondary">Protocol</span>
                        <span className="text-app-text-primary">Uniswap V2</span>
                    </div>
                    <div className="flex justify-between text-xs mt-1">
                        <span className="text-app-text-secondary">Fee Tier</span>
                        <span className="text-app-text-primary">0.30%</span>
                    </div>
                </div>
            </div>
        </div>
    );
}
