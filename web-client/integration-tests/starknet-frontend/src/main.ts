import { starknetRpc } from '@vastrum/react-lib';

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

// Starknet mainnet public RPC
const RPC_URL = 'https://rpc.starknet.lava.build';

// Well-known Starknet mainnet contracts
const ETH_TOKEN = '0x049d36570d4e46f48e99674bd3fcc84644ddd6b96f7c741b1562b82f9e004dc7';

async function testChainId(): Promise<void> {
    console.log('\n[STARKNET] Testing starknet_chainId...');
    const result = await starknetRpc(RPC_URL, 'starknet_chainId', []);
    assertExists(result, 'Chain ID should exist');
    // SN_MAIN is "0x534e5f4d41494e" in hex
    assert(typeof result === 'string', `Chain ID should be a string, got ${typeof result}`);
    assert(result.startsWith('0x'), `Chain ID should be hex, got ${result}`);
    console.log(`   Chain ID: ${result}`);
}

async function testBlockNumber(): Promise<void> {
    console.log('\n[STARKNET] Testing starknet_blockNumber...');
    const result = await starknetRpc(RPC_URL, 'starknet_blockNumber', []);
    assertExists(result, 'Block number should exist');
    assert(typeof result === 'number', `Block number should be a number, got ${typeof result}`);
    assert(result > 0, `Block number should be positive, got ${result}`);
    console.log(`   Block number: ${result}`);
}

async function testGetBlockWithTxHashes(): Promise<void> {
    console.log('\n[STARKNET] Testing starknet_getBlockWithTxHashes...');
    const result = await starknetRpc(RPC_URL, 'starknet_getBlockWithTxHashes', ['latest']);
    assertExists(result, 'Block should exist');
    assertExists(result.block_hash, 'Block hash should exist');
    assertExists(result.block_number, 'Block number should exist');
    assertExists(result.timestamp, 'Timestamp should exist');
    assertExists(result.transactions, 'Transactions should exist');
    assert(Array.isArray(result.transactions), 'Transactions should be an array');
    console.log(`   Block #${result.block_number} hash=${result.block_hash?.substring(0, 18)}... txs=${result.transactions?.length}`);
}

async function testGetNonce(): Promise<void> {
    console.log('\n[STARKNET] Testing starknet_getNonce...');
    const result = await starknetRpc(RPC_URL, 'starknet_getNonce', [
        'latest',
        ETH_TOKEN,
    ]);
    assertExists(result, 'Nonce should exist');
    assert(typeof result === 'string', `Nonce should be a string, got ${typeof result}`);
    console.log(`   ETH token nonce: ${result}`);
}

async function testGetClassHashAt(): Promise<void> {
    console.log('\n[STARKNET] Testing starknet_getClassHashAt...');
    const result = await starknetRpc(RPC_URL, 'starknet_getClassHashAt', [
        'latest',
        ETH_TOKEN,
    ]);
    assertExists(result, 'Class hash should exist');
    assert(typeof result === 'string', `Class hash should be a string, got ${typeof result}`);
    assert(result.startsWith('0x'), `Class hash should be hex, got ${result}`);
    assert(result.length > 10, `Class hash should be non-trivial, got ${result}`);
    console.log(`   ETH token class hash: ${result.substring(0, 18)}...`);
}

async function testGetStorageAt(): Promise<void> {
    console.log('\n[STARKNET] Testing starknet_getStorageAt...');
    // Storage key 0x0 (often the contract name or owner)
    const result = await starknetRpc(RPC_URL, 'starknet_getStorageAt', [
        ETH_TOKEN,
        '0x0',
        'latest',
    ]);
    assertExists(result, 'Storage value should exist');
    assert(typeof result === 'string', `Storage value should be a string, got ${typeof result}`);
    console.log(`   ETH token storage[0x0]: ${result}`);
}

async function testCall(): Promise<void> {
    console.log('\n[STARKNET] Testing starknet_call...');
    // Call name() on ETH token contract
    // name() selector: sn_keccak("name") = 0x361458367e696363fbcc70777d07ebbd2394e89fd0adcaf147faccd1d294d60
    const result = await starknetRpc(RPC_URL, 'starknet_call', [
        {
            contract_address: ETH_TOKEN,
            entry_point_selector: '0x361458367e696363fbcc70777d07ebbd2394e89fd0adcaf147faccd1d294d60',
            calldata: [],
        },
        'latest',
    ]);
    assertExists(result, 'Call result should exist');
    assert(Array.isArray(result), `Call result should be an array, got ${typeof result}`);
    assert(result.length > 0, 'Call result should have at least one element');
    console.log(`   ETH name() result: [${result.join(', ')}]`);
}

async function testGetClassAt(): Promise<void> {
    console.log('\n[STARKNET] Testing starknet_getClassAt...');
    const result = await starknetRpc(RPC_URL, 'starknet_getClassAt', [
        'latest',
        ETH_TOKEN,
    ]);
    assertExists(result, 'Class definition should exist');
    assert(typeof result === 'object', `Class should be an object, got ${typeof result}`);
    // Sierra classes have 'sierra_program', legacy have 'program'
    const hasSierra = 'sierra_program' in result;
    const hasLegacy = 'program' in result;
    assert(hasSierra || hasLegacy, 'Class should have sierra_program or program field');
    console.log(`   ETH token class type: ${hasSierra ? 'Sierra' : 'Legacy'}`);
}

async function run(): Promise<void> {
    console.log('Starting Starknet RPC tests\n');

    try {
        await testChainId();
        await testBlockNumber();
        await testGetBlockWithTxHashes();
        await testGetNonce();
        await testGetClassHashAt();
        await testGetStorageAt();
        await testCall();
        await testGetClassAt();
    } catch (err: any) {
        console.error(`\nFatal error: ${err}\n${err?.stack}`);
        hasFailed = true;
    }

    const status = hasFailed ? 'failed' : 'success';
    console.log(hasFailed ? '\nSome tests failed.' : '\nAll tests passed.');
    window.parent.postMessage({ type: 'test-result', status }, '*');
}

run();
