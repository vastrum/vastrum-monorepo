import { parseAbi, getCreate2Address, keccak256, encodePacked, type Address } from 'viem';
import type { Token, SwapSettings } from './types';

import logoETH from './assets/ETH.png';
import logoWETH from './assets/WETH.png';
import logoUSDC from './assets/USDC.png';
import logoUSDT from './assets/USDT.png';
import logoDAI from './assets/DAI.png';
import logoWBTC from './assets/WBTC.png';
import logoUNI from './assets/UNI.png';
import logoLINK from './assets/LINK.png';
import logoAAVE from './assets/AAVE.png';
import logoCRV from './assets/CRV.png';
import logoMKR from './assets/MKR.png';
import logoSNX from './assets/SNX.png';
import logoYFI from './assets/YFI.png';

export const NATIVE_ETH_ADDRESS = '0x0000000000000000000000000000000000000000' as const;

export const TOKEN_LIST: Token[] = [
    {
        address: NATIVE_ETH_ADDRESS,
        symbol: 'ETH',
        name: 'Ethereum',
        decimals: 18,
        logo: logoETH,
    },
    {
        address: '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2',
        symbol: 'WETH',
        name: 'Wrapped Ether',
        decimals: 18,
        logo: logoWETH,
    },
    {
        address: '0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48',
        symbol: 'USDC',
        name: 'USD Coin',
        decimals: 6,
        logo: logoUSDC,
    },
    {
        address: '0xdac17f958d2ee523a2206206994597c13d831ec7',
        symbol: 'USDT',
        name: 'Tether USD',
        decimals: 6,
        logo: logoUSDT,
    },
    {
        address: '0x6b175474e89094c44da98b954eedeac495271d0f',
        symbol: 'DAI',
        name: 'Dai Stablecoin',
        decimals: 18,
        logo: logoDAI,
    },
    {
        address: '0x2260fac5e5542a773aa44fbcfedf7c193bc2c599',
        symbol: 'WBTC',
        name: 'Wrapped Bitcoin',
        decimals: 8,
        logo: logoWBTC,
    },
    {
        address: '0x1f9840a85d5af5bf1d1762f925bdaddc4201f984',
        symbol: 'UNI',
        name: 'Uniswap',
        decimals: 18,
        logo: logoUNI,
    },
    {
        address: '0x514910771af9ca656af840dff83e8264ecf986ca',
        symbol: 'LINK',
        name: 'Chainlink',
        decimals: 18,
        logo: logoLINK,
    },
    {
        address: '0x7fc66500c84a76ad7e9c93437bfc5ac33e2ddae9',
        symbol: 'AAVE',
        name: 'Aave',
        decimals: 18,
        logo: logoAAVE,
    },
    {
        address: '0xd533a949740bb3306d119cc777fa900ba034cd52',
        symbol: 'CRV',
        name: 'Curve DAO Token',
        decimals: 18,
        logo: logoCRV,
    },
    {
        address: '0x9f8f72aa9304c8b593d555f12ef6589cc3a579a2',
        symbol: 'MKR',
        name: 'Maker',
        decimals: 18,
        logo: logoMKR,
    },
    {
        address: '0xc011a73ee8576fb46f5e1c5751ca3b9fe0af2a6f',
        symbol: 'SNX',
        name: 'Synthetix',
        decimals: 18,
        logo: logoSNX,
    },
    {
        address: '0x0bc529c00c6401aef6d220be8c6ea1667f6ad93e',
        symbol: 'YFI',
        name: 'yearn.finance',
        decimals: 18,
        logo: logoYFI,
    },
];

export const UNISWAP_V2_FACTORY = '0x5c69bee701ef814a2b6a3edd4b1652cb9cc5aa6f' as const;
export const WETH_ADDRESS = '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2' as const;
export const UNISWAP_V2_INIT_CODE_HASH = '0x96e8ac4277198ff8b6f785478aa9a39f403cb768dd02cbee326c3e7da348845f' as const;

export function sortTokens(tokenA: Address, tokenB: Address): [Address, Address] {
    return tokenA.toLowerCase() < tokenB.toLowerCase() ? [tokenA, tokenB] : [tokenB, tokenA];
}

export function computePairAddress(tokenA: Address, tokenB: Address): Address {
    const [token0, token1] = sortTokens(tokenA, tokenB);
    const salt = keccak256(encodePacked(['address', 'address'], [token0, token1]));
    return getCreate2Address({ from: UNISWAP_V2_FACTORY, salt, bytecodeHash: UNISWAP_V2_INIT_CODE_HASH });
}

export const uniswapV2PairAbi = parseAbi([
    'function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)',
    'event Swap(address indexed sender, uint256 amount0In, uint256 amount1In, uint256 amount0Out, uint256 amount1Out, address indexed to)',
]);

export const DEFAULT_SETTINGS: SwapSettings = {
    slippage: '0.5',
    deadline: '20',
};

export const SLIPPAGE_PRESETS = ['0.1', '0.5', '1.0'];
