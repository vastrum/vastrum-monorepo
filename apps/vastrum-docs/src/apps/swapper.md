# Swapper

The usual way of hosting DeFi frontend has some issues.
- There are often many middlemen involved in hosting it. Sometimes the middlemen are down which can bring down the frontend
- Ethereum RPC requests are usually not verified by the frontend and require trusting the RPC provider
- If you choose to self host these services it quickly becomes overwhelming
- If the developer stops maintaining the frontend it usually very quickly disappears, leaving a lost contract very few can interact with

This prototype attempts to fix some of these issues.
                        
- The frontend is stored onchain. The web-client fetchs the frontend from the blockchain and verifies it against the Vastrum state hash merkle tree
- All Ethereum RPC queries are executed using a webbrowser embedded Ethereum Helios light client that verifies the result of all RPC queries. You do not need to trust the RPC provider.

[Swapper](https://uy25lxmolovvfrw6dckfnh3qi4pm2ah5lutj6k2ljqburlbjegta.vastrum.net)

Swapper is a decentralized Uniswap V2 frontend. 
It allows you to.
-   See quotes for trading assets
-   See recent trades in pairs
-   See pair metadata such as reserves


Swapper is implemented using regular ETH RPC tech, all of the data is received by making regular RPC queries to an ETH RPC node.

This is done using the web-client embedded Helios light client to make verified ETH RPC queries.

The Vastrum web-client frontend "runtime" exposes a EIP-1193 Ethereum RPC Javascript provider.

This means you can use any library that supports the EIP-1193 standard. For example viem.


### Swapper example to fetch Uniswap v2 reserves

```javascript
import { createHeliosProvider } from '@vastrum/react-lib';

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


```

Developing the frontend is very similar to developing a regular web2 DeFi frontend. 


## The embedded Helios light client

The Vastrum web-client uses Helios in order to execute trustless Ethereum RPC calls. This works by using some of the Ethereum economic security to verify the current state hash, it then uses some somewhat complicated local function code replay to verify the output of view functions.

[Helios (Github)](https://github.com/a16z/helios)

The web-client connects to the Vastrum RPC node using WebRTC, the Vastrum RPC node supports regular Vastrum keyvalue reads and page reads. However it also supports making ETH RPC requests. The web-client monkeypatches Helios to send its RPC requests to the Vastrum RPC node using WebRTC.

Currently the Vastrum RPC node handles RPC requests by proxying them to an external provider. However in future certain classes of RPC nodes could run an Ethereum node in order to locally respond to ETH RPC requests.


## More exact flow

1. User goes to {SITE_ID}.vastrum.net
2. The web-client frontend is loaded.
3. The web-client connects to a random RPC node using WebRTC-direct (P2P direct IP connection without HTTPS or TLS or domain name requirements)
4. Web-client requests get_page( {SITE_ID} ) from RPC node
5. RPC node generates response + merkle tree proof + most recent consensus certificate
6. Web-client receives response, verifies merkle tree proof + most recent consensus certificate
7. Web-client loads the page/swapper frontend into a sandboxed iframe and it is rendered to the browser
8. Swapper now has code execution inside the sandboxed iframe,
9. Swapper sends a postMessage() request to the parent iframe host containing an ETH RPC request.
10. The web client starts the web-worker for Helios, proxies all RPC requests by Helios to the Vastrum RPC node using WebRTC direct
11. Helios verifies the responses and returns the data to web-client
12. Web-client uses postMessage() to send back the response for the RPC request to Swapper
---
-   The random RPC node is selected from a bootstrap list embedded in web-client (In future could be read from onchain registry)
-   The genesis validator set is also embedded in web-client and is used by the web-client to verify state proofs (currently static validator set)
-   Swapper never has any internet connection or access to anything outside sandboxed iframe, web-client handles all network communication


## Technical implementation

Most of the RPC logic is inside Helios, Vastrum only implements a web-worker for Helios to execute inside and the WebRTC layer for communicating with the Vastrum RPC node.

- Vendored fork of Helios
    -   [vendored-helios (gitter preview)](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum/tree/vendored-helios)
- The webworker host for Helios inside web-client (to prevent freezing main thread)
    - [web-client/helios-worker (gitter preview)](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum/tree/web-client/helios-worker)
- Initialization of the webworker by the web-client
    -   [web-client/app/wasm/src/helios/worker.rs (gitter preview)](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum/tree/web-client/app/wasm/src/helios/worker.rs)
- ETH RPC proxy from vastrum-node RPC
    -   [vastrum-node/src/rpc/handlers/eth_proxy.rs (gitter preview)](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum/tree/vastrum-node/src/rpc/handlers/eth_proxy.rs)



## Things that need to be done


The goal is that any type of DeFi application could be developed and hosted on vastrum, the key missing features currently.
-   Indexing of event data and aggregation of data into more consumable forms, ETH RPC data alone is not enough for most DeFi applications
-   Universal tokenlist solution
-   Faster sync, currently takes 5 seconds to query new state
    -   Directly read from contract state storage instead of using view functions
    -   Make Helios web-worker multithreaded   
-   Incentivized RPC node running by having users mine POW hash to pay for RPC node services (mining subsidized by Vastrum network)
-   Currently you cannot sign or create any transactions, just read data from Ethereum.


## Why

The hope is by developing a credible full stack alternative to hosting DeFi frontends you achieve this
-   True decentralization
-   True sovereignty
-   Much easier to deploy a new dApp, deploying the frontend is just like deploying the smart contract
-   No recurring hosting cost and operational complexity, just dev and single deploy and done
-   Smart contracts are currently very hard to censor, it is however very easy to censor frontends. By having the blockchain host the frontend also you achieve full stack censorship resistance.
-   True DAO ownership, DAO ownership has never been credible because the centralized labs always controlled the frontend even if they did not control the smart contract, by having a fully decentralized stack the hope is that you could make a fully functioning credible DAO with actual ownership and control of the protocol

[Swapper on Gitter](https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/repo/vastrum/tree/apps/swapper)


