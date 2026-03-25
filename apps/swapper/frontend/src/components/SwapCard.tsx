import { useState, useEffect, useCallback, useMemo } from 'react';
import { formatUnits } from 'viem';
import { SwapIcon, SettingsIcon } from './Icons';
import { TokenInput } from './TokenInput';
import { TokenSelectorModal } from './TokenSelectorModal';
import { SettingsModal } from './SettingsModal';
import { WETH_ADDRESS, NATIVE_ETH_ADDRESS } from '../constants';
import type { SwapState, PairInfo, SwapSettings } from '../types';

interface SwapCardProps {
    swapState: SwapState;
    setSwapState: React.Dispatch<React.SetStateAction<SwapState>>;
    pairInfo: PairInfo | null;
    onSwap: () => void;
    isLoading: boolean;
    settings: SwapSettings;
    setSettings: React.Dispatch<React.SetStateAction<SwapSettings>>;
}

export function SwapCard({
    swapState,
    setSwapState,
    pairInfo,
    onSwap,
    isLoading,
    settings,
    setSettings,
}: SwapCardProps) {
    const [selectorOpen, setSelectorOpen] = useState<'input' | 'output' | null>(null);
    const [settingsOpen, setSettingsOpen] = useState(false);

    const { rate, outputAmount } = useMemo(() => {
        if (!pairInfo || !swapState.inputAmount) {
            return { rate: null, outputAmount: '' };
        }

        const inputNum = parseFloat(swapState.inputAmount);
        if (isNaN(inputNum) || inputNum <= 0) {
            return { rate: null, outputAmount: '' };
        }

        const inputAddress = swapState.inputToken.address === NATIVE_ETH_ADDRESS ? WETH_ADDRESS : swapState.inputToken.address;
        const inputIsToken0 = inputAddress.toLowerCase() === pairInfo.token0.toLowerCase();

        const token0Decimals = inputIsToken0 ? swapState.inputToken.decimals : swapState.outputToken.decimals;
        const token1Decimals = inputIsToken0 ? swapState.outputToken.decimals : swapState.inputToken.decimals;

        const reserve0Formatted = Number(formatUnits(pairInfo.reserve0, token0Decimals));
        const reserve1Formatted = Number(formatUnits(pairInfo.reserve1, token1Decimals));

        const inputReserve = inputIsToken0 ? reserve0Formatted : reserve1Formatted;
        const outputReserve = inputIsToken0 ? reserve1Formatted : reserve0Formatted;

        const inputWithFee = inputNum * 997;
        const numerator = inputWithFee * outputReserve;
        const denominator = (inputReserve * 1000) + inputWithFee;
        const output = numerator / denominator;

        const calculatedRate = outputReserve / inputReserve;

        const formatAmount = (n: number, decimals: number) => {
            const fixed = n.toFixed(Math.min(decimals, 6));
            return fixed.includes('.') ? fixed.replace(/0+$/, '').replace(/\.$/, '') : fixed;
        };

        return {
            rate: calculatedRate,
            outputAmount: formatAmount(output, swapState.outputToken.decimals),
        };
    }, [pairInfo, swapState.inputAmount, swapState.inputToken, swapState.outputToken]);

    useEffect(() => {
        if (outputAmount !== swapState.outputAmount) {
            setSwapState((prev) => ({ ...prev, outputAmount }));
        }
    }, [outputAmount, setSwapState, swapState.outputAmount]);

    const handleInputAmountChange = useCallback((value: string) => {
        if (value === '' || /^\d*\.?\d*$/.test(value)) {
            setSwapState((prev) => ({
                ...prev,
                inputAmount: value,
            }));
        }
    }, [setSwapState]);

    const handleSwapTokens = useCallback(() => {
        setSwapState((prev) => ({
            inputToken: prev.outputToken,
            outputToken: prev.inputToken,
            inputAmount: '',
            outputAmount: '',
        }));
    }, [setSwapState]);

    const handleTokenSelect = useCallback((token: typeof swapState.inputToken) => {
        if (selectorOpen === 'input') {
            setSwapState((prev) => ({
                ...prev,
                inputToken: token,
                inputAmount: '',
                outputAmount: '',
            }));
        } else if (selectorOpen === 'output') {
            setSwapState((prev) => ({
                ...prev,
                outputToken: token,
                inputAmount: '',
                outputAmount: '',
            }));
        }
    }, [selectorOpen, setSwapState]);

    const hasValidInput = swapState.inputAmount && parseFloat(swapState.inputAmount) > 0;

    return (
        <>
            <div className="bg-app-bg-secondary border border-app-border rounded-3xl p-4 w-full max-w-[480px]">
                <div className="flex justify-between items-center mb-4">
                    <h2 className="text-app-text-primary text-xl font-semibold">Swap</h2>
                    <button
                        onClick={() => setSettingsOpen(true)}
                        className="text-app-text-secondary hover:text-app-text-primary transition-colors p-2"
                    >
                        <SettingsIcon />
                    </button>
                </div>

                <TokenInput
                    label="You pay"
                    token={swapState.inputToken}
                    amount={swapState.inputAmount}
                    onAmountChange={handleInputAmountChange}
                    onTokenClick={() => setSelectorOpen('input')}
                />

                <div className="flex justify-center -my-2 relative z-10">
                    <button
                        onClick={handleSwapTokens}
                        className="bg-app-bg-secondary border-4 border-app-bg-primary rounded-xl p-2 hover:bg-app-bg-tertiary transition-colors"
                    >
                        <SwapIcon />
                    </button>
                </div>

                <TokenInput
                    label="You receive"
                    token={swapState.outputToken}
                    amount={swapState.outputAmount}
                    onTokenClick={() => setSelectorOpen('output')}
                    readOnly
                />

                {rate && pairInfo && (
                    <div className="mt-4 p-3 bg-app-bg-tertiary rounded-xl space-y-2">
                        <div className="flex justify-between text-sm">
                            <span className="text-app-text-secondary">Rate</span>
                            <span className="text-app-text-primary">
                                1 {swapState.inputToken.symbol} = {rate.toFixed(6)} {swapState.outputToken.symbol}
                            </span>
                        </div>
                        <div className="flex justify-between text-sm">
                            <span className="text-app-text-secondary">Price Impact</span>
                            <span className="text-app-text-primary">
                                {swapState.inputAmount && parseFloat(swapState.inputAmount) > 0
                                    ? `${(((rate - (parseFloat(swapState.outputAmount) / parseFloat(swapState.inputAmount))) / rate) * 100).toFixed(2)}%`
                                    : '0.00%'}
                            </span>
                        </div>
                        <div className="flex justify-between text-sm">
                            <span className="text-app-text-secondary">LP Fee</span>
                            <span className="text-app-text-primary">0.30%</span>
                        </div>
                        <div className="flex justify-between text-sm">
                            <span className="text-app-text-secondary">Max Slippage</span>
                            <span className="text-app-text-primary">{settings.slippage}%</span>
                        </div>
                    </div>
                )}

                <button
                    onClick={onSwap}
                    disabled={!hasValidInput || isLoading || !pairInfo}
                    className={`w-full mt-4 py-4 rounded-2xl font-semibold text-lg transition-all ${hasValidInput && !isLoading && pairInfo
                        ? 'bg-app-accent-blue hover:bg-app-accent-blue/90 text-white cursor-pointer'
                        : 'bg-app-bg-tertiary text-app-text-secondary cursor-not-allowed'
                        }`}
                >
                    {isLoading
                        ? 'Loading... (This might take 10-20s)'
                        : !pairInfo
                            ? 'No liquidity pool found'
                            : hasValidInput
                                ? 'Swap'
                                : 'Enter an amount'}
                </button>
            </div>

            <TokenSelectorModal
                isOpen={selectorOpen !== null}
                onClose={() => setSelectorOpen(null)}
                onSelect={handleTokenSelect}
                selectedToken={selectorOpen === 'input' ? swapState.inputToken : swapState.outputToken}
                otherToken={selectorOpen === 'input' ? swapState.outputToken : swapState.inputToken}
            />

            <SettingsModal
                isOpen={settingsOpen}
                onClose={() => setSettingsOpen(false)}
                settings={settings}
                onSettingsChange={setSettings}
            />
        </>
    );
}
