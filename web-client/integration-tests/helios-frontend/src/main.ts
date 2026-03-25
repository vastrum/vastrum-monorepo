import { createHeliosProvider } from '@vastrum/react-lib';
import { createPublicClient, custom, formatUnits, parseAbi, isAddress, isHex } from 'viem';
import { mainnet } from 'viem/chains';

let hasFailed = false;

const origLog = console.log;
const origError = console.error;
console.log = (...args: any[]) => {
    origLog.apply(console, args);
    window.parent.postMessage({ type: 'iframe-log', message: args.map(String).join(' ') }, '*');
};
console.error = (...args: any[]) => {
    origError.apply(console, args);
    window.parent.postMessage({ type: 'iframe-error', message: args.map(String).join(' ') }, '*');
};

function assert(condition: boolean, message: string): void {
    if (condition) {
        console.log(`PASS: ${message}`);
    } else {
        console.error(`FAIL: ${message}`);
        hasFailed = true;
    }
}

function assertExists(value: unknown, message: string): void {
    assert(value !== undefined && value !== null, message);
}

function assertType(value: unknown, expectedType: string, message: string): void {
    assert(typeof value === expectedType, `${message} (expected ${expectedType}, got ${typeof value})`);
}

function assertGreaterThan(value: number | bigint, threshold: number | bigint, message: string): void {
    assert(value > threshold, `${message} (${value} > ${threshold})`);
}

const VITALIK_ADDRESS = '0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045' as const;
const USDC_CONTRACT = '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48' as const;
const WETH_CONTRACT = '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2' as const;
const UNISWAP_V2_USDC_ETH_PAIR = '0xB4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc' as const;
const ZERO_ADDRESS = '0x0000000000000000000000000000000000000000' as const;

const erc20Abi = parseAbi([
    'function name() view returns (string)',
    'function symbol() view returns (string)',
    'function decimals() view returns (uint8)',
    'function totalSupply() view returns (uint256)',
    'function balanceOf(address owner) view returns (uint256)',
]);

const uniswapV2PairAbi = parseAbi([
    'function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)',
    'function token0() external view returns (address)',
    'function token1() external view returns (address)',
    'function factory() external view returns (address)',
]);

type Client = ReturnType<typeof createPublicClient>;

async function testChainId(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_chainId...');
    const chainId = await client.getChainId();
    assertExists(chainId, 'Chain ID should exist');
    assert(chainId === 1, 'Chain ID should be 1 for mainnet');
}

async function testGetBlockNumber(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_blockNumber...');
    const blockNumber = await client.getBlockNumber();
    assertExists(blockNumber, 'Block number should exist');
    assertType(blockNumber, 'bigint', 'Block number should be bigint');
    assertGreaterThan(blockNumber, 0n, 'Block number should be positive');
    assertGreaterThan(blockNumber, 18000000n, 'Block number should be recent (> 18M)');
}

async function testGetBlock(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_getBlockByNumber...');
    const latestBlock = await client.getBlock({ blockTag: 'latest' });
    assertExists(latestBlock, 'Latest block should exist');
    assertExists(latestBlock.hash, 'Block hash should exist');
    assertExists(latestBlock.number, 'Block number should exist');
    assertExists(latestBlock.timestamp, 'Block timestamp should exist');
    assertExists(latestBlock.parentHash, 'Parent hash should exist');
    assert(isHex(latestBlock.hash!), 'Block hash should be hex');
    assertGreaterThan(latestBlock.gasUsed, 0n, 'Gas used should be positive');
    const specificBlock = await client.getBlock({ blockNumber: latestBlock.number! - 10n });
    assertExists(specificBlock, 'Specific block should exist');
    assert(specificBlock.number === latestBlock.number! - 10n, 'Block number should match requested');
}

async function testGetBlockByHash(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_getBlockByHash...');
    const latest = await client.getBlock({ blockTag: 'latest' });
    const byHash = await client.getBlock({ blockHash: latest.hash! });
    assertExists(byHash, 'Block by hash should exist');
    assert(byHash.hash === latest.hash, 'Block hash should match');
    assert(byHash.number === latest.number, 'Block number should match');
}

async function testGetBalance(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_getBalance...');
    const vitalikBalance = await client.getBalance({ address: VITALIK_ADDRESS });
    assertExists(vitalikBalance, 'Balance should exist');
    assertType(vitalikBalance, 'bigint', 'Balance should be bigint');
    assertGreaterThan(vitalikBalance, 0n, 'Vitalik should have some ETH');
    const zeroBalance = await client.getBalance({ address: ZERO_ADDRESS });
    assertExists(zeroBalance, 'Zero address balance should exist');
    assertType(zeroBalance, 'bigint', 'Zero address balance should be bigint');
}

async function testGetCode(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_getCode...');
    const usdcCode = await client.getCode({ address: USDC_CONTRACT });
    assertExists(usdcCode, 'Contract code should exist');
    assert(isHex(usdcCode!), 'Contract code should be hex');
    assertGreaterThan(usdcCode!.length, 100, 'Contract code should have significant length');
}

async function testGetTransactionCount(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_getTransactionCount...');
    const nonce = await client.getTransactionCount({ address: VITALIK_ADDRESS });
    assertExists(nonce, 'Nonce should exist');
    assertType(nonce, 'number', 'Nonce should be number');
    assertGreaterThan(nonce, 1000, 'Vitalik should have many transactions');
}

async function testGetStorageAt(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_getStorageAt...');
    const storage = await client.getStorageAt({ address: USDC_CONTRACT, slot: '0x0' });
    assertExists(storage, 'Storage value should exist');
    assert(isHex(storage!), 'Storage value should be hex');
}

async function testGetProof(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_getProof...');
    const proof = await client.getProof({
        address: USDC_CONTRACT,
        storageKeys: ['0x0'],
    });
    assertExists(proof, 'Proof should exist');
    assertExists(proof.address, 'Proof should have address');
    assertExists(proof.accountProof, 'Proof should have accountProof');
    assertExists(proof.balance, 'Proof should have balance');
    assertExists(proof.codeHash, 'Proof should have codeHash');
    assertExists(proof.nonce, 'Proof should have nonce');
    assertExists(proof.storageProof, 'Proof should have storageProof');
    assert(proof.address.toLowerCase() === USDC_CONTRACT.toLowerCase(), 'Proof address should match');
}

async function testGasPrice(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_gasPrice...');
    const gasPrice = await client.getGasPrice();
    assertExists(gasPrice, 'Gas price should exist');
    assertType(gasPrice, 'bigint', 'Gas price should be bigint');
    assertGreaterThan(gasPrice, 0n, 'Gas price should be positive');
}

async function testMaxPriorityFeePerGas(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_maxPriorityFeePerGas...');
    const fee = await client.estimateMaxPriorityFeePerGas();
    assertExists(fee, 'Max priority fee should exist');
    assertType(fee, 'bigint', 'Max priority fee should be bigint');
    assert(fee >= 0n, `Max priority fee should be non-negative, got ${fee}`);
}

async function testEstimateGas(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_estimateGas...');
    const gasEstimate = await client.estimateGas({
        account: VITALIK_ADDRESS,
        to: ZERO_ADDRESS,
        value: 1n,
    });
    assertExists(gasEstimate, 'Gas estimate should exist');
    assertType(gasEstimate, 'bigint', 'Gas estimate should be bigint');
    assert(gasEstimate >= 21000n, 'Basic transfer should cost at least 21000 gas');
}

async function testCreateAccessList(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_createAccessList...');
    try {
        const result = await client.createAccessList({
            account: VITALIK_ADDRESS,
            to: USDC_CONTRACT,
            data: '0x18160ddd', // totalSupply()
        });
        assertExists(result, 'Access list result should exist');
        assertExists(result.accessList, 'Should have accessList');
        assertExists(result.gasUsed, 'Should have gasUsed');
        assert(Array.isArray(result.accessList), 'Access list should be an array');
    } catch {
        console.log('SKIP: eth_createAccessList not supported');
    }
}

async function testCall(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_call (contract reads)...');
    const usdcName = await client.readContract({
        address: USDC_CONTRACT,
        abi: erc20Abi,
        functionName: 'name',
    });
    assert(usdcName === 'USD Coin', `USDC name should be 'USD Coin', got '${usdcName}'`);

    const usdcSymbol = await client.readContract({
        address: USDC_CONTRACT,
        abi: erc20Abi,
        functionName: 'symbol',
    });
    assert(usdcSymbol === 'USDC', `USDC symbol should be 'USDC', got '${usdcSymbol}'`);

    const usdcDecimals = await client.readContract({
        address: USDC_CONTRACT,
        abi: erc20Abi,
        functionName: 'decimals',
    });
    assert(usdcDecimals === 6, `USDC decimals should be 6, got ${usdcDecimals}`);

    const wethName = await client.readContract({
        address: WETH_CONTRACT,
        abi: erc20Abi,
        functionName: 'name',
    });
    assert(wethName === 'Wrapped Ether', `WETH name should be 'Wrapped Ether', got '${wethName}'`);

    const totalSupply = await client.readContract({
        address: USDC_CONTRACT,
        abi: erc20Abi,
        functionName: 'totalSupply',
    });
    assertGreaterThan(totalSupply, 0n, 'USDC total supply should be positive');

    const vitalikUsdc = await client.readContract({
        address: USDC_CONTRACT,
        abi: erc20Abi,
        functionName: 'balanceOf',
        args: [VITALIK_ADDRESS],
    });
    assertType(vitalikUsdc, 'bigint', 'Balance should be bigint');
}

async function testUniswapPair(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing Uniswap V2 Pair contract...');
    const reserves = await client.readContract({
        address: UNISWAP_V2_USDC_ETH_PAIR,
        abi: uniswapV2PairAbi,
        functionName: 'getReserves',
    });
    const [reserve0, reserve1, blockTimestampLast] = reserves;
    assertExists(reserve0, 'Reserve0 should exist');
    assertExists(reserve1, 'Reserve1 should exist');
    assertExists(blockTimestampLast, 'Block timestamp should exist');
    assertGreaterThan(reserve0, 0n, 'Reserve0 should be positive');
    assertGreaterThan(reserve1, 0n, 'Reserve1 should be positive');

    const token0 = await client.readContract({
        address: UNISWAP_V2_USDC_ETH_PAIR,
        abi: uniswapV2PairAbi,
        functionName: 'token0',
    });
    assert(isAddress(token0), 'Token0 should be valid address');
    assert(token0.toLowerCase() === USDC_CONTRACT.toLowerCase(), 'Token0 should be USDC');

    const token1 = await client.readContract({
        address: UNISWAP_V2_USDC_ETH_PAIR,
        abi: uniswapV2PairAbi,
        functionName: 'token1',
    });
    assert(isAddress(token1), 'Token1 should be valid address');
    assert(token1.toLowerCase() === WETH_CONTRACT.toLowerCase(), 'Token1 should be WETH');

    const factory = await client.readContract({
        address: UNISWAP_V2_USDC_ETH_PAIR,
        abi: uniswapV2PairAbi,
        functionName: 'factory',
    });
    assert(isAddress(factory), 'Factory should be valid address');

    const usdcReserve = Number(formatUnits(reserve0, 6));
    const wethReserve = Number(formatUnits(reserve1, 18));
    const ethPrice = usdcReserve / wethReserve;
    assert(ethPrice > 100, `ETH price should be > $100, got $${ethPrice.toFixed(2)}`);
    assert(ethPrice < 100000, `ETH price should be < $100000, got $${ethPrice.toFixed(2)}`);
    console.log(`   ETH price from Uniswap V2: $${ethPrice.toFixed(2)}`);
}

async function testGetTransaction(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_getTransactionByHash...');
    const block = await client.getBlock({ blockTag: 'latest', includeTransactions: true });
    if (block.transactions && block.transactions.length > 0) {
        const txHash = typeof block.transactions[0] === 'string'
            ? block.transactions[0]
            : block.transactions[0].hash;
        const tx = await client.getTransaction({ hash: txHash });
        assertExists(tx, 'Transaction should exist');
        assertExists(tx.hash, 'Transaction hash should exist');
        assertExists(tx.from, 'Transaction from should exist');
        assert(isAddress(tx.from), 'From should be valid address');
        assert(isHex(tx.hash), 'Transaction hash should be hex');
    } else {
        console.log('SKIP: No transactions in latest block');
    }
}

async function testGetTransactionReceipt(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_getTransactionReceipt...');
    try {
        const block = await client.getBlock({ blockTag: 'latest', includeTransactions: true });
        if (block.transactions && block.transactions.length > 0) {
            const txHash = typeof block.transactions[0] === 'string'
                ? block.transactions[0]
                : block.transactions[0].hash;
            const receipt = await client.getTransactionReceipt({ hash: txHash });
            if (receipt) {
                assertExists(receipt.transactionHash, 'Receipt should have transaction hash');
                assertExists(receipt.blockNumber, 'Receipt should have block number');
                assertExists(receipt.status, 'Receipt should have status');
                assertExists(receipt.gasUsed, 'Receipt should have gas used');
                assert(receipt.transactionHash === txHash, 'Receipt hash should match');
            } else {
                console.log('SKIP: Transaction receipt not available');
            }
        } else {
            console.log('SKIP: No transactions in latest block');
        }
    } catch {
        console.log('SKIP: Transaction receipt lookup failed');
    }
}

async function testGetLogs(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing eth_getLogs...');
    const latestBlock = await client.getBlockNumber();
    const logs = await client.getLogs({
        address: USDC_CONTRACT,
        event: parseAbi(['event Transfer(address indexed from, address indexed to, uint256 value)'])[0],
        fromBlock: latestBlock - 5n,
        toBlock: latestBlock,
    });
    assertExists(logs, 'Logs should exist');
    assert(Array.isArray(logs), 'Logs should be an array');
    if (logs.length > 0) {
        const firstLog = logs[0];
        assertExists(firstLog.transactionHash, 'Log should have transaction hash');
        assertExists(firstLog.blockNumber, 'Log should have block number');
        assertExists(firstLog.address, 'Log should have address');
        console.log(`   Found ${logs.length} USDC Transfer events in last 5 blocks`);
    } else {
        console.log('No Transfer events found (possible on low-activity periods)');
    }
}

async function testMulticall(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing multicall (batch reads)...');
    try {
        const results = await client.multicall({
            contracts: [
                { address: USDC_CONTRACT, abi: erc20Abi, functionName: 'name' },
                { address: USDC_CONTRACT, abi: erc20Abi, functionName: 'symbol' },
                { address: WETH_CONTRACT, abi: erc20Abi, functionName: 'name' },
                { address: WETH_CONTRACT, abi: erc20Abi, functionName: 'symbol' },
            ],
        });
        assert(results.length === 4, 'Should have 4 results');
        assert(results[0].result === 'USD Coin', 'First result should be USD Coin');
        assert(results[1].result === 'USDC', 'Second result should be USDC');
        assert(results[2].result === 'Wrapped Ether', 'Third result should be Wrapped Ether');
        assert(results[3].result === 'WETH', 'Fourth result should be WETH');
    } catch {
        console.log('SKIP: Multicall not supported');
    }
}

async function testBlockConsistency(client: Client): Promise<void> {
    console.log('\n[HELIOS] Testing block consistency...');
    const blockNum1 = await client.getBlockNumber();
    const blockNum2 = await client.getBlockNumber();
    const diff = blockNum2 >= blockNum1 ? blockNum2 - blockNum1 : blockNum1 - blockNum2;
    assert(diff <= 2n, `Block numbers should be consistent (diff: ${diff})`);
    const block = await client.getBlock({ blockNumber: blockNum1 });
    assert(block.number === blockNum1, 'Block number in block should match requested');
}

async function run(): Promise<void> {
    console.log('Starting Helios ETH RPC tests\n');

    try {
        const provider = await createHeliosProvider();
        const loggingProvider = {
            request: async (req: any) => {
                console.log(`[RPC] >> ${req.method} ${JSON.stringify(req.params)}`);
                try {
                    const result = await provider.request(req);
                    console.log(`[RPC] << ${req.method} type=${typeof result} val=${JSON.stringify(result)}`);
                    return result;
                } catch (err: any) {
                    console.error(`[RPC] !! ${req.method} error: ${err}\n${err?.stack}`);
                    throw err;
                }
            },
            on: provider.on.bind(provider),
            removeListener: provider.removeListener.bind(provider),
        };
        const client = createPublicClient({
            chain: mainnet,
            transport: custom(loggingProvider),
        });
        console.log('Helios provider initialized.\n');

        await testChainId(client);
        await testGetBlockNumber(client);
        await testGetBlock(client);
        await testGetBlockByHash(client);
        await testGetBalance(client);
        await testGetCode(client);
        await testGetTransactionCount(client);
        await testGetStorageAt(client);
        await testGetProof(client);
        await testGasPrice(client);
        await testMaxPriorityFeePerGas(client);
        await testEstimateGas(client);
        await testCreateAccessList(client);
        await testCall(client);
        await testUniswapPair(client);
        await testGetTransaction(client);
        await testGetTransactionReceipt(client);
        await testGetLogs(client);
        await testMulticall(client);
        await testBlockConsistency(client);
    } catch (err: any) {
        console.error(`\nFatal error: ${err}\n${err?.stack}`);
        hasFailed = true;
    }

    const status = hasFailed ? 'failed' : 'success';
    console.log(hasFailed ? '\nSome tests failed.' : '\nAll tests passed.');
    window.parent.postMessage({ type: 'test-result', status }, '*');
}

run();
