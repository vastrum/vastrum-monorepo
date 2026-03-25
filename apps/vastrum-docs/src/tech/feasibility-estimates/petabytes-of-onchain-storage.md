# Petabytes of Onchain Storage

The idea is to have Vastrum be capable of hosting petabytes+ onchain. This probably the feature with the highest amount of technical risk, it is not yet solved. The plan is very handwavy as there is not yet a concrete implementation, but i think it is solvable problem.

## Storing petabytes onchain has two problem
-   Need to store petabytes
-   Also need to give client read access to petabytes with web2 like latency


## Storing petabytes

You can not require validator nodes to store petabytes, if you did basically nobody would be able to run a validator node.

I think a sensible storage requirement level to be somewhere between 1-10 TB per validator node.

### Solution 1: Execution shards

Split applications up into execution shards, validator nodes only need to store the state for that specific site. This scales nicely but it limits single application state storage limits to 1-10 TB.


### Solution 2: Write only writes (blobs)

The validator node needs to store the complete keyvalue state of each application because the application expects to be able to read from any of it.

However you can introduce a new type of keyvalue write type, a "write only" write. The application cannot read this value after it is written.

This means the validator node can discard the state after execution, this means application "state" could increase to petabyte levels as most of the state does not have to be stored by the validator nodes.

#### Concrete examples
-   5 MB/s of blockspace
-   Potentially 432 GB of data written per day
-   13 TB of state increase per month
-   However if the majority of writes are "write only" then maybe 431 GB of state is discarded by the validator nodes and only 1 GB is kept per day
-   Now you only have 30 GB of state increase per month


#### Usecases for this type of write
-   Gitter git objects do not need to be read by the contract, validator nodes should not have to keep it in state.
-   Chatter messages also does not need to be read by contract
-   Forum posts could be made constant size and only contain a SHA256 digest with the underlying post text content being stored in write only storage


#### Data availability for write only writes

Having validators discard valid application state creates data availability problems.

There are multiple ways of solving this.
-   Have each validator in the execution shard share data availability burden, ie if 10 validator nodes inside a execution shard then each node stores 20% of the write only data. 
-   Have special data availability nodes which focus on storage over execution, this could work with small and large nodes so could have nodes with 100 GB of storage and mega nodes with 100 TB+

There is also some design dynamics
-   What is the physical storage amplification factor
    -   This has major impact on cost structure
    -   Can 2 GB of physical storage be used across network for 1 GB of actual stored data?
    -   Web2 cloud can often achieve very small amplification < 2, why cant this be achieved for web3?


This also interacts with RPC nodes and how you make the data available to clients.
-   How is the data transferred to RPC nodes?
-   How is the data transferred to data availability nodes and how do they signal a complete storage? 
    -   Should they be apart of consensus where consensus is halted if DA fails to keep up with chain?


#### Stateless execution

This data should still be in the state merkle tree proof held by validator nodes, so could do stateless execution where clients provide the data + proofs so that the contracts could still use the data if needed.








## Reading petabytes


The other major problem is how do you provide access to petabytes of blockchain data?


RPC nodes are intended to be decentralized nodes which can be ran on commodity servers.

Same restrictions applies here as they do to validator nodes, max 1-10 TB of storage use.

This means that the client needs to somehow interact with multiple RPC nodes to satisfy request on petabyte size datasets.

There are many ways of doing this.
-   Each RPC node only stores part of the total blockchain state
    -   This could be per site for example
    -   But some sites might have states in the 100s of TB, so would still need to connect to many RPC nodes for a single site
    -   This would mean the web-client would need to connect to potentially a very large amount of RPC nodes
    -   Could have proxy RPC node architecture where the client connects to a single RPC node which then itself queries other RPC nodes to answer requests.

-   Each RPC node stores reed-solomon encoded shards of total blockchain storage, client reconstructs
    -   Same problem of having to connect to many RPC nodes, could do same proxy RPC architecture here to solve this.

It is probably fine to have the web-client connect to 10 RPC nodes directly.


I do not have a concrete architecture to solve this problem, but it does not seem to be something that would be impossible to solve. The hardest problem is probably the economic incentives. You need to somehow convince many operators to run RPC nodes with substantial storage costs.

## Economic incentives to run RPC nodes

Alternatives to solve this
-   DAO pays operators of RPC nodes
    -   Problem is no guarantees operators actually operate RPC node and answer queries.
        -   Have some centralized operator test if the RPC nodes are actually answering queries, similar to how TOR checks the bandwidth provided by each router
    -   Commission more trusted orgs/foundations to run RPC nodes? Could be credibly censorship resistant if have 50 - 100 different ideological entities

-   Ideological altruism :)
    -   Many people run TOR nodes

-   Native client runs a RPC node to answer queries for other clients in background, ie regular decentralized internet P2P solution
    -   Privacy issues
    -   Bandwidth use issues
    -   Latency issues
    -   No hard limits that data you are interested in is actually available


-   Pay RPC node with actual token micro payments
    -   Limits read access of network to onboarded token holders which is opposite to whole architecture of Vastrum


-   Client mines for RPC node, RPC node get rewarded by the mined POW
    -   Sub electricity cost POW subsidised by network
    -   No miners will mine the POW if reward is less than electrical cost
    -   Only used as value transfer function from client to RPC node
    -   RPC node acts as mining pool, if meet difficulty submits hash to network and receives reward
    -   Problem if get electricity cost wrong then expose token minting through mining



### Client mines for RPC node, RPC node get rewarded by the mined POW

This would be the ideal solution. For the RPC node you also get built in application layer DDOS resistance, ie reject queries which do not contain valid POW.


For the DOS resistance model i used these value for what each device could mine.

    phone 0,0001 USD / hour
    laptop/desktop = 0.001 USD / hour


#### What is the cost structure of running a RPC node?

Assume most of the cost is bandwidth.

With Hetzner you pay roughly 1 USD per TB of outgoing bandwidth.


This means a phone can pay for

    0,0001 * 1 TB = 100 MB reads per hour

Computer
    
    0,001 = 1 GB reads per hour




#### This is not very optimal. Preferably would like to have 100x these values.

One solution could be to have a kind of market pricing of RPC servers, perhaps some RPC providers would accept lower pricing because they have access to cheaper outgoing bandwidth. 

Maybe some operators accept some loss as long as at least some of their loss is subsidised.

Another solution is to increase network subsidies, maybe even above power cost and only decrease it if large scale mining operations start.

Many people have > 10 MB/s of unlimited outgoing bandwidth in their home connection that is mostly unused and "free", perhaps they could host a RPC node at home, this would probably require some proxy or running node behind TOR to avoid leaking home IP.

At full utilization a 10 MB/s connection would have roughly 1 TB of sent bandwidth per day, this is roughly 1 dollar in revenue per day utilizing a "free" resource.


Ultimately the best solution will probably be a combination of all above. The optimal solution will also change with the maturity of the project and how hardware evolves in the future.