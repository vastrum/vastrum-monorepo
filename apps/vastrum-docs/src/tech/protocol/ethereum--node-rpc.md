# Ethereum Node RPC

Currently Vastrum provides full stack ETH RPC access in the frontend runtime
-   Helios light client embedded in web-client verifies ETH RPC requests
-   Vastrum RPC node serves ETH RPC requests by web-client.


Currently the Vastrum RPC node only proxies RPC requests to external provider, eventually will self host a full node. 

Swapper is an experimental Uniswap v2 frontend hosted on Vastrum that uses this feature.

There is much more information in the technical docs of Swapper how the ETH RPC integration works.

More info in [Swapper docs](../../apps/swapper.md)


Future work
-   Most DeFi application use indexing services beyond what RPC nodes can provide, similar functionality should be natively supported

