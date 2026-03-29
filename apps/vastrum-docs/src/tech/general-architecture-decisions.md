# Onchain Maximalism

Doing everything onchain goes against all common sense instinct of how to construct a decentralized P2P website hosting system, however it is the only way to achieve Web2 usability, latency and availability. 

The major problem with doing P2P swarm like systems
-   Latency, have to search deep within network to find desired data
-   No data availability guarantees, most systems are lossy or evicts old data
-   No native economic model, relies on altruistic nodes.

By doing everything onchain with execution shards
-   Latency is "constant"
-   Hard availability guarantees, your data will never be pruned
-   Can construct a sustainable economic model for node operators


By doing it onchain you can also delegate some of the effort of maintaining the system to node operators, 
the selfhosted path would be optimal where all business host their own website on their own server instead of Facebook. However most users are not willing to operate their own servers.

The internet is by default decentralized, everybody could run their own Git servers or HTTP servers. But the effort required to run your own server is fairly high. This means in practice most users choose to use centralized platforms like Github because of their reliability and ease of use.

 To achieve the same level of accessibility in a decentralized system, you have to somehow construct a system where no regular user is required to self host. A "decentralized Github" instead of a "decentralized set of Git servers".




## Scaling throughput

It would never be possible to host the totality of the internet on a single blockchain shard, for scaling there are two approaches.

### Execution sharding

A credibly decentralized blockchain cluster could realistically at most push 1-5 MB of block throughput per second and maybe 10k-20k TPS.

This is not enough, to drive more throughput sharding has to be implemented.

Calls between sites are not supported, this makes it very easy to put each site into its own execution shard.


This means protocol can horizontally scale well, however vertical scaling is still limited to 1 MB/s and 10k TPS per site.


[Execution Sharding docs](../roadmap/execution-sharding.md) has more information.

### Write only writes (Blobs)

Frontier blockchain consensus algorithms can support roughly 1-5 MB of data throughput per second per shard. However if this data is written into state then blockchain state grows at 432 GB per day. This is not sustainable.

Most of the data writes are of the type where it is not of interest by the smart contract.

For example for Gitter (Github clone) git objects, execution does not need to have access to it, git objects represent > 99.9% of total storage needs of Gitter.

Write only writes are not stored by validators and the data is discarded after execution, instead the data is stored by data availability nodes and made available to clients by RPC nodes.

This would, for example, allow making most interaction with Gitter work just on proof of work without requiring users to use gas tokens.

You can still do processing on the data within the block the transaction is received.

For example you can calculate the SHA1 hash of the git object inside the smart contract.

You just cannot read the data from the smart contract later. Only client keyvalue reads have access to it through RPC nodes.

This should allow for petabyte scale onchain storage.

[Write-Only Writes docs](../roadmap/writeonly-writes.md) has more information.


## Web-client for web2 website accessibility parity

One choice would be to develop a native "web3 browser" for exploring the Vastrum network, however to ensure maximum accessibility a gateway web-client accessible from a regular web browser was chosen instead.

In the future perhaps a native Vastrum browser will be implemented, 
it could basically just be a chromium web browser using current web-client code.

This would eliminate the frontend centralization risks of the web-client hosted on vastrum.net. But requires onboarding users to install it.

## No wallet

Another choice to ensure web2 UX parity was to not require an external wallet, 
the private key is stored by the web-client in localstorage.

Onboarding users to a wallet extension is simpler than onboarding them to a new web3 browser however it still requires significant investment from the user.

By allowing users to directly interact with Vastrum network using just a web browser, accessibility is improved.

The major problem is that it is easy to lose access to your account, 
hopefully a decentralized ZK email based social recovery solution can solve this partly.


## No gas token payments

Another obstacle to interacting with blockchains is the gas token payment, 
it is not feasible to expect users to pay to make a forum post or similar.

However it is also not feasible to not have any form of protocol layer spam protection.

The middle ground to this is a POW based gas payment. 

Every transaction executed has to compute a small POW puzzle, this gives some level of economic cost to spamming the network with transactions.

However the cost created by this is still very small, a spammer can still send a massive amount of unproductive transactions.

To solve this you have to scale the protocol immensely,

The primary reason for expanding the TPS handled by the network is to enable this POW gas architecture to work.

POW gas would never work with 100 TPS or even 10k TPS, with 1 million TPS (across all shards) the DOS costs becomes reasonably high enough.

If there is credible DOS resistance, the network will not be DOSed.

For now the network can only handle roughly 10k TPS but hopefully applications can be split across multiple execution shards to increase TPS.

[DOS Resistance docs](feasibility-estimates/dos-resistance.md) has more information.

## The web-client page runtime 

The current architecture is SPA applications with all HTML+JS+CSS+WASM embedded directly into a single html file.

These then have full JS execution capability inside a sandboxed iframe.

The sandboxed iframe blocks all external networking requests.

All external communication is handled through the postMessage() IFrame API

The application can make a keyvalue read or submit a transaction by sending a message through this API to the host web-client.

The web-client manages the connection to the RPC node using webRTC and proxies the keyvalue read request or the transaction submission.