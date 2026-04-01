For this hackathon i implemented a Starknet light client.

I have previously developed Swapper for Vastrum which was a Uniswap V2 frontend hosted on Vastrum using the Helios light client embedded inside the web browser to do verified RPC queries. 

I did this by forking Beerus which is an abandoned project for creating a webbrowser embeddable light client for Starknet.
https://github.com/vastrum/Beeruser


I then integrated this with the vastrum web-client to create starknet-frontend.vastrum.net. This DEX frontend verifies all RPC queries using state proofs.


I started this work in the last couple hours so i did not have time to properly implement it. Current problems
 

 -  Swapper uses WebRTC to directly connect to RPC node for Helios requests, starknet uses HTTP requests.
- I could not find a good way to verify the state hash for Starknet, you could read the L1 state hash but that would be severely delayed. So in the current implementation you trust the RPC client for the state hash, however all requests are verified against that state hash. If starknet implements state hash verification for light clients then it would be a full stack light client.



More information is in the docs if interested.


Other site examples hosted on Vastrum:


Decentralized Github

https://yts27rvo7ppzq5rrjyavmfwecrbyc5ksldmitiggycetgh6zguoa.vastrum.net/

Docs

https://docs.vastrum.net/introduction