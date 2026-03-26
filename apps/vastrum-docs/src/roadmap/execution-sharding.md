# Execution Sharding

The most important feature to scale the network is execution sharding. Instead of having global state and global execution, split the validator set into multiple smaller sets which each have responsibility of doing consensus + execution for a subset of sites.

Full parallelism between sites is possible because all smart contracts are isolated from each other, cross smart contract calling is not supported.


Having execution shards reduces the economic security though, this makes it much easier to fool clients into accept invalid state such as phishing pages or similar.

The idea to solve this is to have global HTML/page state. All pages of websites are guaranteed by the full economic security of the network.

This works because.
-   Pages are very rarely updated
    -   perhaps you could limit page updates to deploy time only
-   Most of the compute is assigned to keyvalue operations
-   Most of the benefits of execution sharding comes from sharding keyvalue operations

This increases security significantly, even if you have control of an execution shard you cannot manipulate the pages. 

This limits the type of attacks that you can do.


## Chatter
-   Encrypted keyvalue store, malicious execution shard can only censor, cannot manipulate messages

## Gitter
-   Malicious execution shard could set a malicious head commit id on any repository 
    -   Perhaps global state for repo name ownership + signed updates?
    -   Or repo name = public key
-   For git objects, if client verifies the SHA1 hash of the objects, execution shard can only censor.

## Concord
-   Encrypted messages, can confuse clients with metadata though


## Concourse
-   Can manipulate posts



Most of these manipulations should be detectable relatively quickly and should be possible to slash. 

The biggest gain is by eliminating phishing pages as an attack vector most of the incentive to 66% attack execution shards disappears. It is not viable to attempt to replace a DeFi frontend with a phishing page because that would require 66% attack on the total economic security of the network, not just 66% within the execution shard the DeFi frontend is hosted in.
