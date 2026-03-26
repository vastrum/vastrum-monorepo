import type { Address } from 'viem';

export interface Token {
    address: Address;
    symbol: string;
    name: string;
    decimals: number;
    logo: string;
}

export interface SwapState {
    inputToken: Token;
    outputToken: Token;
    inputAmount: string;
    outputAmount: string;
}

export interface PairInfo {
    pairAddress: Address;
    reserve0: bigint;
    reserve1: bigint;
    token0: Address;
    token1: Address;
}

export interface SwapSettings {
    slippage: string;
    deadline: string;
}

export interface Trade {
    txHash: string;
    blockNumber: bigint;
    amount0In: bigint;
    amount1In: bigint;
    amount0Out: bigint;
    amount1Out: bigint;
    sender: string;
    to: string;
    timestamp?: number;
}
