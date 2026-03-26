import { useState, useEffect, useRef } from 'react';
import { createPublicClient, custom } from 'viem';
import { mainnet } from 'viem/chains';
import {
    SwapCard,
    ConnectionStatus,
    PoolSidebar,
    WelcomeModal,
    Modal,
    RecentTrades,
} from './components';
import {
    TOKEN_LIST,
    WETH_ADDRESS,
    NATIVE_ETH_ADDRESS,
    uniswapV2PairAbi,
    DEFAULT_SETTINGS,
    computePairAddress,
    sortTokens,
} from './constants';
import type { SwapState, PairInfo, SwapSettings, Trade } from './types';
import './styles/index.css';
import { createHeliosProvider } from '@vastrum/react-lib';


function App() {
    const [swapState, setSwapState] = useState<SwapState>({
        inputToken: TOKEN_LIST[0],
        outputToken: TOKEN_LIST[2],
        inputAmount: '',
        outputAmount: '',
    });

    const [pairInfo, setPairInfo] = useState<PairInfo | null>(null);
    const [isConnected, setIsConnected] = useState(false);
    const [isLoading, setIsLoading] = useState(true);
    const [blockNumber, setBlockNumber] = useState<bigint | null>(null);
    const [recentTrades, setRecentTrades] = useState<Trade[]>([]);
    const [tradesLoading, setTradesLoading] = useState(false);
    const [settings, setSettings] = useState<SwapSettings>(DEFAULT_SETTINGS);
    const [showWelcome, setShowWelcome] = useState(true);
    const [showSwapNotImplemented, setShowSwapNotImplemented] = useState(false);
    const lastFetchedBlockRef = useRef<bigint | null>(null);

    useEffect(() => {
        if (!pairInfo) {
            setRecentTrades([]);
            lastFetchedBlockRef.current = null;
            return;
        }

        const pairAddress = pairInfo.pairAddress;
        let mounted = true;

        function logsToTrades(logs: any[]): Trade[] {
            return logs.map((log) => ({
                txHash: log.transactionHash || '',
                blockNumber: log.blockNumber || 0n,
                amount0In: (log.args as any).amount0In || 0n,
                amount1In: (log.args as any).amount1In || 0n,
                amount0Out: (log.args as any).amount0Out || 0n,
                amount1Out: (log.args as any).amount1Out || 0n,
                sender: (log.args as any).sender || '',
                to: (log.args as any).to || '',
            }));
        }

        async function fetchTrades(fromBlock: bigint, toBlock: bigint) {
            const provider = createHeliosProvider();
            const viemClient = createPublicClient({
                chain: mainnet,
                transport: custom(provider),
            });
            return viemClient.getLogs({
                address: pairAddress,
                event: uniswapV2PairAbi[1],
                fromBlock,
                toBlock,
            });
        }

        async function fetchInitialTrades() {
            setTradesLoading(true);
            try {
                const provider = createHeliosProvider();
                const viemClient = createPublicClient({
                    chain: mainnet,
                    transport: custom(provider),
                });
                const currentBlock = await viemClient.getBlockNumber();
                const fromBlock = currentBlock - 32n;

                const logs = await fetchTrades(fromBlock, currentBlock);
                if (!mounted) return;

                const trades = logsToTrades(logs).reverse();
                setRecentTrades(trades.slice(0, 50));
                lastFetchedBlockRef.current = currentBlock;
            } catch (err) {
                console.error('Error fetching initial trades:', err);
                if (mounted) setRecentTrades([]);
            } finally {
                if (mounted) setTradesLoading(false);
            }
        }

        async function fetchNewTrades() {
            try {
                const provider = createHeliosProvider();
                const viemClient = createPublicClient({
                    chain: mainnet,
                    transport: custom(provider),
                });
                const currentBlock = await viemClient.getBlockNumber();
                const lastBlock = lastFetchedBlockRef.current;

                if (!lastBlock || currentBlock - lastBlock > 32n) {
                    await fetchInitialTrades();
                    return;
                }

                if (currentBlock <= lastBlock) return;

                const logs = await fetchTrades(lastBlock + 1n, currentBlock);
                if (!mounted) return;

                if (logs.length > 0) {
                    const newTrades = logsToTrades(logs);
                    setRecentTrades(prev => {
                        const existingHashes = new Set(prev.map(t => t.txHash));
                        const unique = newTrades.filter(t => !existingHashes.has(t.txHash));
                        return [...unique.reverse(), ...prev].slice(0, 50);
                    });
                }
                lastFetchedBlockRef.current = currentBlock;
            } catch (err) {
                console.error('Error fetching new trades:', err);
            }
        }

        fetchInitialTrades();
        const intervalId = setInterval(fetchNewTrades, 15000);

        return () => {
            mounted = false;
            lastFetchedBlockRef.current = null;
            clearInterval(intervalId);
        };
    }, [pairInfo?.pairAddress]);

    useEffect(() => {
        let intervalId: ReturnType<typeof setInterval>;
        let retryTimeoutId: ReturnType<typeof setTimeout>;
        let mounted = true;
        let consecutiveFailures = 0;

        async function fetchPairInfo() {
            try {
                const provider = createHeliosProvider();
                const viemClient = createPublicClient({
                    chain: mainnet,
                    transport: custom(provider),
                });

                const inputAddress = swapState.inputToken.address === NATIVE_ETH_ADDRESS ? WETH_ADDRESS : swapState.inputToken.address;
                const outputAddress = swapState.outputToken.address === NATIVE_ETH_ADDRESS ? WETH_ADDRESS : swapState.outputToken.address;

                const pairAddress = computePairAddress(inputAddress, outputAddress);
                const [token0, token1] = sortTokens(inputAddress, outputAddress);

                const reserves = await viemClient.readContract({
                    address: pairAddress,
                    abi: uniswapV2PairAbi,
                    functionName: 'getReserves',
                });

                if (!mounted) return;

                if (reserves[0] === 0n && reserves[1] === 0n) {
                    setPairInfo(null);
                    setIsLoading(false);
                    setIsConnected(true);
                    consecutiveFailures = 0;
                    return;
                }

                setPairInfo({
                    pairAddress,
                    reserve0: reserves[0],
                    reserve1: reserves[1],
                    token0,
                    token1,
                });

                setIsConnected(true);
                setIsLoading(false);
                consecutiveFailures = 0;
            } catch (err) {
                console.error('Error fetching pair info:', err);
                if (mounted) {
                    consecutiveFailures++;
                    setIsConnected(false);

                    const retryDelay = Math.min(2000 * Math.pow(2, consecutiveFailures - 1), 30000);
                    retryTimeoutId = setTimeout(() => {
                        if (mounted) {
                            fetchPairInfo();
                        }
                    }, retryDelay);

                    if (consecutiveFailures === 1) {
                        setIsLoading(false);
                    }
                }
            }
        }

        setPairInfo(null);
        setIsLoading(true);
        fetchPairInfo();

        intervalId = setInterval(fetchPairInfo, 10000);

        return () => {
            mounted = false;
            clearInterval(intervalId);
            clearTimeout(retryTimeoutId);
        };
    }, [swapState.inputToken, swapState.outputToken]);

    // Poll for new blocks to show liveness
    useEffect(() => {
        let mounted = true;
        let intervalId: ReturnType<typeof setInterval>;

        async function pollBlockNumber() {
            try {
                const provider = createHeliosProvider();
                const viemClient = createPublicClient({
                    chain: mainnet,
                    transport: custom(provider),
                });

                const latestBlock = await viemClient.getBlockNumber();

                if (mounted) {
                    setBlockNumber(latestBlock);
                    setIsConnected(true);
                }
            } catch (err) {
                console.error('Block poll error:', err);
            }
        }

        pollBlockNumber();
        intervalId = setInterval(pollBlockNumber, 12000);

        return () => {
            mounted = false;
            clearInterval(intervalId);
        };
    }, []);

    const handleSwap = () => {
        setShowSwapNotImplemented(true);
    };

    return (
        <div className="min-h-screen bg-app-bg-primary flex flex-col">
            <header className="border-b border-app-border px-6 py-4">
                <div className="max-w-7xl mx-auto flex justify-between items-center">
                    <div className="flex items-center gap-2">
                        <span className="text-app-accent-blue text-2xl font-bold">Swapper</span>
                    </div>
                    <ConnectionStatus isConnected={isConnected} isLoading={isLoading} blockNumber={blockNumber} />
                </div>
            </header>

            <main className="flex-1 flex flex-col items-center p-6 gap-6 pt-12">
                <div className="flex flex-col lg:flex-row items-center lg:items-start justify-center gap-6 w-full max-w-5xl">
                    <SwapCard
                        swapState={swapState}
                        setSwapState={setSwapState}
                        pairInfo={pairInfo}
                        onSwap={handleSwap}
                        isLoading={isLoading}
                        settings={settings}
                        setSettings={setSettings}
                    />
                    <PoolSidebar
                        pairInfo={pairInfo}
                        inputToken={swapState.inputToken}
                        outputToken={swapState.outputToken}
                        isLoading={isLoading}
                        blockNumber={blockNumber}
                    />
                </div>

                {pairInfo && (() => {
                    const inputIsToken0 = (swapState.inputToken.address === NATIVE_ETH_ADDRESS ? WETH_ADDRESS : swapState.inputToken.address).toLowerCase() === pairInfo.token0.toLowerCase();
                    return (
                        <div className="w-full max-w-[320px] md:max-w-5xl">
                            <RecentTrades
                                trades={recentTrades}
                                token0={inputIsToken0 ? swapState.inputToken : swapState.outputToken}
                                token1={inputIsToken0 ? swapState.outputToken : swapState.inputToken}
                                token0Decimals={inputIsToken0 ? swapState.inputToken.decimals : swapState.outputToken.decimals}
                                token1Decimals={inputIsToken0 ? swapState.outputToken.decimals : swapState.inputToken.decimals}
                                isLoading={tradesLoading}
                            />
                        </div>
                    );
                })()}
            </main>

            <footer className="border-t border-app-border px-6 py-4">

            </footer>

            <WelcomeModal isOpen={showWelcome} onClose={() => setShowWelcome(false)} />

            <Modal isOpen={showSwapNotImplemented} onClose={() => setShowSwapNotImplemented(false)} title="Not Implemented">
                <div className="p-6 space-y-4">
                    <p className="text-app-text-secondary">
                        Submitting transactions to Ethereum is not currently supported. Only reading from Ethereum is currently implemented.
                    </p>
                    <div className="flex justify-end pt-2">
                        <button
                            onClick={() => setShowSwapNotImplemented(false)}
                            className="bg-app-accent-green text-white px-4 py-2 rounded-md font-medium hover:bg-[#2ea043] transition-colors"
                        >
                            Close
                        </button>
                    </div>
                </div>
            </Modal>
        </div>
    );
}

export default App;
