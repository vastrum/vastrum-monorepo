import { useState, useEffect } from 'react';
import { createPublicClient, custom, parseAbi, getCreate2Address, keccak256, encodePacked, formatUnits, type Address } from 'viem';
import { mainnet } from 'viem/chains';
import { RouterProvider, Outlet, createMemoryRouter } from 'react-router-dom';
import './styles/index.css';
import { createVastrumReactRouter, createHeliosProvider } from '@vastrum/react-lib';

const UNISWAP_V2_FACTORY = '0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f' as const;
const WETH = '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2' as const;
const USDC = '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48' as const;
const INIT_CODE_HASH = '0x96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f' as const;

const pairAbi = parseAbi([
    'function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)',
]);

function computePairAddress(tokenA: Address, tokenB: Address): Address {
    const [token0, token1] = tokenA.toLowerCase() < tokenB.toLowerCase() ? [tokenA, tokenB] : [tokenB, tokenA];
    const salt = keccak256(encodePacked(['address', 'address'], [token0, token1]));
    return getCreate2Address({ from: UNISWAP_V2_FACTORY, salt, bytecodeHash: INIT_CODE_HASH });
}

function Layout() {
    return (
        <div className="min-h-screen bg-app-bg text-app-text">
            <div className="max-w-xl mx-auto px-4 py-8">
                <Outlet />
            </div>
        </div>
    );
}

function Home() {
    const [blockNumber, setBlockNumber] = useState<bigint | null>(null);
    const [isConnected, setIsConnected] = useState(false);
    const [reserves, setReserves] = useState<{ weth: bigint; usdc: bigint } | null>(null);

    useEffect(() => {
        let cancelled = false;
        let reserveInterval: ReturnType<typeof setInterval>;
        let blockInterval: ReturnType<typeof setInterval>;

        async function init() {
            const provider = createHeliosProvider();
            const client = createPublicClient({
                chain: mainnet,
                transport: custom(provider),
            });

            if (cancelled) return;
            setIsConnected(true);

            const pairAddress = computePairAddress(WETH, USDC);
            const isWethToken0 = WETH.toLowerCase() < USDC.toLowerCase();

            const fetchReserves = async () => {
                if (cancelled) return;
                try {
                    const result = await client.readContract({
                        address: pairAddress,
                        abi: pairAbi,
                        functionName: 'getReserves',
                    });
                    if (!cancelled) {
                        setReserves({
                            weth: isWethToken0 ? result[0] : result[1],
                            usdc: isWethToken0 ? result[1] : result[0],
                        });
                    }
                } catch (err) {
                    console.error('Reserve fetch error:', err);
                }
            };

            const fetchBlock = async () => {
                if (cancelled) return;
                try {
                    const bn = await client.getBlockNumber();
                    if (!cancelled) setBlockNumber(bn);
                } catch (err) {
                    console.error('Block fetch error:', err);
                }
            };

            fetchReserves();
            fetchBlock();
            reserveInterval = setInterval(fetchReserves, 10_000);
            blockInterval = setInterval(fetchBlock, 12_000);
        }

        init();
        return () => {
            cancelled = true;
            clearInterval(reserveInterval);
            clearInterval(blockInterval);
        };
    }, []);

    const spotPrice = reserves
        ? Number(formatUnits(reserves.usdc, 6)) / Number(formatUnits(reserves.weth, 18))
        : null;

    return (
        <div className="flex flex-col gap-6">
            <h1 className="text-3xl font-bold">{"{{Name}}"}</h1>

            <div className="bg-app-surface border border-app-border rounded-lg p-5 flex flex-col gap-4">
                <div className="flex justify-between items-center">
                    <h2 className="text-lg font-semibold">WETH / USDC Pool</h2>
                    <span className={`text-xs px-2 py-1 rounded ${isConnected ? 'bg-green-900 text-green-300' : 'bg-yellow-900 text-yellow-300'}`}>
                        {isConnected ? 'Connected' : 'Connecting...'}
                    </span>
                </div>

                {reserves ? (
                    <div className="flex flex-col gap-2">
                        <div className="flex justify-between">
                            <span className="text-app-text-secondary">WETH Reserve</span>
                            <span className="font-mono">{Number(formatUnits(reserves.weth, 18)).toFixed(4)}</span>
                        </div>
                        <div className="flex justify-between">
                            <span className="text-app-text-secondary">USDC Reserve</span>
                            <span className="font-mono">{Number(formatUnits(reserves.usdc, 6)).toFixed(2)}</span>
                        </div>
                        {spotPrice !== null && (
                            <div className="flex justify-between border-t border-app-border pt-2 mt-1">
                                <span className="text-app-text-secondary">Price</span>
                                <span className="font-mono">{spotPrice.toFixed(2)} USDC/ETH</span>
                            </div>
                        )}
                    </div>
                ) : (
                    <p className="text-app-text-muted text-sm">Loading reserves...</p>
                )}
            </div>

            <div className="bg-app-surface border border-app-border rounded-lg p-5">
                <div className="flex justify-between items-center">
                    <span className="text-app-text-secondary text-sm">Ethereum Block</span>
                    <span className="font-mono text-lg">
                        {blockNumber !== null ? `#${blockNumber.toString()}` : '...'}
                    </span>
                </div>
            </div>

        </div>
    );
}

const routes = [
    {
        element: <Layout />,
        children: [
            { path: '/', element: <Home /> },
        ],
    },
];

export const router = await createVastrumReactRouter(routes, createMemoryRouter);

export default function App() {
    return <RouterProvider router={router} />;
}
