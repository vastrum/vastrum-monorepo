import './styles/index.css';
import { useState, useEffect, useCallback } from 'react';
import { starknetRpc } from '@vastrum/react-lib';

import logoETH from './assets/ETH.png';
import logoUSDC from './assets/USDC.png';
import logoSTRK from './assets/STRK.png';
import logoWBTC from './assets/WBTC.png';
import logoEKUBO from './assets/EKUBO.png';

const DEFAULT_RPC = 'https://rpc.starknet.lava.build';
const EKUBO_CORE = '0x00000005dd3D2F4429AF886cD1a3b08289DBcEa99A294197E9eB43b0e0325b4b';

interface Token { address: string; decimals: number; symbol: string; name: string; logo?: string }

const TOKEN_LIST: Token[] = [
    { address: '0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7', decimals: 18, symbol: 'ETH',     name: 'Ether',          logo: logoETH },
    { address: '0x053c91253bc9682c04929ca02ed00b3e423f6710d2ee7e0d5ebb06f3ecf368a8', decimals: 6,  symbol: 'USDC',    name: 'USD Coin',       logo: logoUSDC },
    { address: '0x04718f5a0fc34cc1af16a1cdee98ffb20c31f5cd61d6ab07201858f4287c938d', decimals: 18, symbol: 'STRK',    name: 'Starknet Token', logo: logoSTRK },
    { address: '0x03fe2b97c1fd336e750087d68b9b867997fd64a2661ff3ca5a7c771641e8e7ac', decimals: 8,  symbol: 'WBTC',    name: 'Wrapped Bitcoin', logo: logoWBTC },
    { address: '0x04daa17763b286d1e59b97c283c0b8c949994c361e426a28f743c67bdfe9a32f', decimals: 18, symbol: 'tBTC',    name: 'Threshold BTC' },
    { address: '0x0593e034dda23eea82d2ba9a30960ed42cf4a01502cc2351dc9b9881f9931a68', decimals: 18, symbol: 'SolvBTC', name: 'Solv BTC' },
    { address: '0x036834a40984312f7f7de8d31e3f6305b325389eaeea5b1c0664b2fb936461a4', decimals: 8,  symbol: 'LBTC',    name: 'Lombard BTC' },
    { address: '0x075afe6402ad5a5c20dd25e10ec3b3986acaa647b77e4ae24b0cbc9a54a27a87', decimals: 18, symbol: 'EKUBO',   name: 'Ekubo Protocol', logo: logoEKUBO },
];

const SEL = {
    balance_of:     '0x35a73cd311a05d46deda634c5ee045db92f811b4e74bca4437fcb5302b7af33',
    get_pool_price: '0x63ecb4395e589622a41a66715a0eac930abc9f0b92c0b1dcda630adfb2bf2d',
};

const FEE = '0x20c49ba5e353f80000000000000000';
const TICK_SP = '0x3e8';
const FEE_PCT = 0.0005;

const FALLBACK_COLORS: Record<string, string> = {
    ETH: '#627eea', USDC: '#2775ca', STRK: '#ff6b35', WBTC: '#f09242',
    tBTC: '#1a1a2e', SolvBTC: '#f59e0b', LBTC: '#e11d48', EKUBO: '#7c3aed',
};

function feltToU256(low: string, high: string): bigint {
    return BigInt(low) + (BigInt(high) << 128n);
}

function fmt(n: number, d = 2): string {
    if (n >= 1e6) return (n / 1e6).toFixed(d) + 'M';
    if (n >= 1e3) return (n / 1e3).toFixed(d) + 'K';
    if (n >= 1) return n.toFixed(d);
    if (n >= 0.0001) return n.toFixed(6);
    return n.toExponential(3);
}

function sortPair(a: Token, b: Token): [Token, Token] {
    return BigInt(a.address) < BigInt(b.address) ? [a, b] : [b, a];
}

function TokenIcon({ token, size = 24 }: { token: Token; size?: number }) {
    if (token.logo) {
        return <img src={token.logo} alt={token.symbol} className="rounded-full shrink-0" style={{ width: size, height: size }} />;
    }
    const abbr = token.symbol.length <= 4 ? token.symbol.slice(0, 2) : token.symbol[0];
    return (
        <div className="rounded-full flex items-center justify-center text-white font-bold shrink-0"
            style={{ width: size, height: size, background: FALLBACK_COLORS[token.symbol] || '#555', fontSize: size * 0.35 }}>
            {abbr}
        </div>
    );
}

function TokenButton({ token, onClick }: { token: Token; onClick: () => void }) {
    return (
        <button onClick={onClick}
            className="cursor-pointer flex items-center gap-2 px-3 py-2 rounded-2xl hover:bg-[#353840] transition-colors shrink-0"
            style={{ background: '#2c2f36' }}>
            <TokenIcon token={token} size={24} />
            <span className="text-white font-semibold text-base">{token.symbol}</span>
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none" className="text-[#8f96ac]">
                <path d="M3 4.5L6 7.5L9 4.5" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round"/>
            </svg>
        </button>
    );
}

function TokenModal({ open, onClose, onSelect }: {
    open: boolean; onClose: () => void; onSelect: (t: Token) => void;
}) {
    if (!open) return null;
    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center" onClick={onClose}>
            <div className="absolute inset-0 bg-black/60" />
            <div className="relative w-full max-w-[360px] mx-4 rounded-2xl bg-[#212429] border border-[#2c2f36] overflow-hidden"
                onClick={e => e.stopPropagation()}>
                <div className="flex items-center justify-between px-5 py-4 border-b border-[#2c2f36]">
                    <span className="text-white font-medium">Select a token</span>
                    <button onClick={onClose} className="cursor-pointer text-[#8f96ac] hover:text-white text-xl leading-none">&times;</button>
                </div>
                <div className="flex flex-wrap gap-2 px-4 pt-3 pb-2">
                    {TOKEN_LIST.slice(0, 4).map(t => (
                        <button key={t.symbol} onClick={() => { onSelect(t); onClose(); }}
                            className="cursor-pointer flex items-center gap-1.5 px-3 py-1.5 rounded-xl border border-[#2c2f36] hover:bg-[#2c2f36] transition-colors">
                            <TokenIcon token={t} size={20} />
                            <span className="text-white text-sm font-medium">{t.symbol}</span>
                        </button>
                    ))}
                </div>
                <div className="border-t border-[#2c2f36] mx-4" />
                <div className="p-2 max-h-[320px] overflow-y-auto">
                    {TOKEN_LIST.map(t => (
                        <button key={t.symbol} onClick={() => { onSelect(t); onClose(); }}
                            className="cursor-pointer w-full flex items-center gap-3 px-4 py-3 rounded-xl hover:bg-[#2c2f36] transition-colors">
                            <TokenIcon token={t} size={36} />
                            <div className="text-left flex-1">
                                <div className="text-white font-medium">{t.symbol}</div>
                                <div className="text-[#8f96ac] text-xs">{t.name}</div>
                            </div>
                        </button>
                    ))}
                </div>
            </div>
        </div>
    );
}

function Row({ label, value, white, valueColor }: { label: string; value: string; white?: boolean; valueColor?: string }) {
    return (
        <div className="flex justify-between text-[#8f96ac]">
            <span>{label}</span>
            <span style={{ color: valueColor || (white ? '#fff' : '#8f96ac') }}>{value}</span>
        </div>
    );
}

function App() {
    const [rpcUrl, setRpcUrl] = useState(DEFAULT_RPC);
    const [showSettings, setShowSettings] = useState(false);
    const [fromToken, setFromToken] = useState(TOKEN_LIST[0]);
    const [toToken, setToToken] = useState(TOKEN_LIST[1]);
    const [modal, setModal] = useState<'from' | 'to' | null>(null);
    const [inputAmt, setInputAmt] = useState('');
    const [pool, setPool] = useState<{ price: number; tick: number; bal0: number; bal1: number } | null>(null);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState('');

    const [t0, t1] = sortPair(fromToken, toToken);
    const fromIsT0 = fromToken.symbol === t0.symbol;

    const fetchPool = useCallback(async () => {
        setLoading(true);
        setError('');
        try {
            const [pr, b0, b1] = await Promise.all([
                starknetRpc(rpcUrl, 'starknet_call', [{
                    contract_address: EKUBO_CORE,
                    entry_point_selector: SEL.get_pool_price,
                    calldata: [t0.address, t1.address, FEE, TICK_SP, '0x0'],
                }, 'latest']),
                starknetRpc(rpcUrl, 'starknet_call', [{
                    contract_address: t0.address,
                    entry_point_selector: SEL.balance_of,
                    calldata: [EKUBO_CORE],
                }, 'latest']),
                starknetRpc(rpcUrl, 'starknet_call', [{
                    contract_address: t1.address,
                    entry_point_selector: SEL.balance_of,
                    calldata: [EKUBO_CORE],
                }, 'latest']),
            ]);
            const sr = Number(feltToU256(pr[0], pr[1])) / Number(2n ** 128n);
            const price = sr * sr * 10 ** (t0.decimals - t1.decimals);
            const tick = Number(BigInt(pr[2])) * (Number(BigInt(pr[3])) === 1 ? -1 : 1);
            const bal0 = Number(feltToU256(b0[0], b0[1])) / 10 ** t0.decimals;
            const bal1 = Number(feltToU256(b1[0], b1[1])) / 10 ** t1.decimals;
            setPool({ price, tick, bal0, bal1 });
        } catch (e: any) {
            setError(e?.message || String(e));
            setPool(null);
        } finally {
            setLoading(false);
        }
    }, [rpcUrl, t0, t1]);

    useEffect(() => { fetchPool(); }, [fetchPool]);

    // pool.price = t1_per_t0 (decimal-adjusted)
    // displayPrice = toToken per fromToken
    const displayPrice = pool ? (fromIsT0 ? pool.price : 1 / pool.price) : null;

    // Swap quote: use spot price from sqrt_ratio (not global balances which span all pools)
    let outputAmt: number | null = null;
    if (pool && inputAmt && Number(inputAmt) > 0 && displayPrice) {
        outputAmt = Number(inputAmt) * displayPrice * (1 - FEE_PCT);
    }

    const handleSelectToken = (t: Token) => {
        if (modal === 'from') {
            if (t.symbol === toToken.symbol) setToToken(fromToken);
            setFromToken(t);
        } else {
            if (t.symbol === fromToken.symbol) setFromToken(toToken);
            setToToken(t);
        }
        setInputAmt('');
        setModal(null);
    };

    const flip = () => { setFromToken(toToken); setToToken(fromToken); setInputAmt(''); };

    const fromBal = pool ? (fromIsT0 ? pool.bal0 : pool.bal1) : null;
    const toBal = pool ? (fromIsT0 ? pool.bal1 : pool.bal0) : null;

    return (
        <div className="min-h-screen bg-[#191b1f] flex flex-col items-center pt-12 px-4">
            <div className="w-full max-w-[420px] flex items-center justify-between mb-3">
                <span className="text-white text-lg font-semibold">Swap</span>
                <button onClick={() => setShowSettings(!showSettings)}
                    className="cursor-pointer w-8 h-8 rounded-lg flex items-center justify-center text-[#8f96ac] hover:text-white hover:bg-[#2c2f36] transition-colors">
                    <svg width="18" height="18" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" viewBox="0 0 24 24">
                        <circle cx="12" cy="12" r="3"/><path d="M12 1v2m0 18v2m-9.07-2.93 1.41-1.41m12.73-12.73 1.41-1.41M1 12h2m18 0h2m-2.93 9.07-1.41-1.41M4.34 4.34 2.93 2.93"/>
                    </svg>
                </button>
            </div>

            {showSettings && (
                <div className="w-full max-w-[420px] mb-3 rounded-2xl bg-[#212429] border border-[#2c2f36] p-4">
                    <label className="text-xs text-[#8f96ac] mb-1 block">RPC Endpoint</label>
                    <input className="w-full bg-[#191b1f] border border-[#2c2f36] rounded-xl px-3 py-2 text-sm text-white outline-none focus:border-[#e8590c]"
                        value={rpcUrl} onChange={e => setRpcUrl(e.target.value)} />
                </div>
            )}

            <div className="w-full max-w-[420px] rounded-2xl bg-[#212429] border border-[#2c2f36] p-3">
                <div className="rounded-2xl bg-[#191b1f] p-4 mb-1">
                    <div className="flex justify-between mb-2">
                        <span className="text-xs text-[#8f96ac]">From</span>
                        {fromBal !== null && <span className="text-xs text-[#8f96ac]">Pool: {fmt(fromBal)} {fromToken.symbol}</span>}
                    </div>
                    <div className="flex items-center gap-3">
                        <input type="text" inputMode="decimal" placeholder="0" value={inputAmt}
                            onChange={e => { if (e.target.value === '' || /^\d*\.?\d*$/.test(e.target.value)) setInputAmt(e.target.value); }}
                            className="flex-1 text-3xl font-medium bg-transparent text-white outline-none min-w-0" />
                        <TokenButton token={fromToken} onClick={() => setModal('from')} />
                    </div>
                </div>

                <div className="flex justify-center -my-3 relative z-10">
                    <button onClick={flip}
                        className="cursor-pointer w-10 h-10 rounded-xl bg-[#212429] border-4 border-[#191b1f] flex items-center justify-center text-[#8f96ac] hover:text-white transition-colors">
                        <svg width="16" height="16" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" viewBox="0 0 24 24">
                            <path d="M7 4v16m0 0-4-4m4 4 4-4M17 20V4m0 0 4 4m-4-4-4 4"/>
                        </svg>
                    </button>
                </div>

                <div className="rounded-2xl bg-[#191b1f] p-4 mt-1">
                    <div className="flex justify-between mb-2">
                        <span className="text-xs text-[#8f96ac]">To (estimated)</span>
                        {toBal !== null && <span className="text-xs text-[#8f96ac]">Pool: {fmt(toBal)} {toToken.symbol}</span>}
                    </div>
                    <div className="flex items-center gap-3">
                        <div className="flex-1 text-3xl font-medium min-w-0" style={{ color: outputAmt ? '#fff' : '#5d6785' }}>
                            {outputAmt !== null ? fmt(outputAmt, 4) : '0'}
                        </div>
                        <TokenButton token={toToken} onClick={() => setModal('to')} />
                    </div>
                </div>

                {pool && displayPrice !== null && (
                    <div className="mt-3 rounded-xl bg-[#191b1f] px-4 py-3 space-y-2 text-sm">
                        <Row label="Price" value={`1 ${fromToken.symbol} = ${fmt(displayPrice, 4)} ${toToken.symbol}`} />
                        <Row label="Note" value="Spot price quote (no slippage model)" />
                        <Row label="Fee" value="0.05%" />
                        <Row label="Tick" value={pool.tick.toLocaleString()} white />
                    </div>
                )}

                <div className="mt-3">
                    {error && <div className="text-red-400 text-xs mb-2 break-all">{error}</div>}
                    <button onClick={fetchPool} disabled={loading}
                        className="cursor-pointer w-full py-3.5 rounded-2xl text-base font-semibold transition-colors"
                        style={{ background: loading ? '#2c2f36' : '#e8590c', color: '#fff' }}>
                        {loading ? 'Loading...' : 'Refresh Price'}
                    </button>
                </div>
            </div>

            <div className="text-[#5d6785] text-xs mt-4">Ekubo Protocol on Starknet</div>

            <TokenModal open={modal !== null} onClose={() => setModal(null)} onSelect={handleSelectToken} />
        </div>
    );
}

export default App;
