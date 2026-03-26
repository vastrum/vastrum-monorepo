# Private Information Retrieval

Private information retrieval (PIR) is a cryptographic technique which allows you to request information from a server without revealing what information you accessed.

For Vastrum specifically it would look like this
-   Currently a RPC node can see everything clients requests
-   Using PIR, the RPC node could only see that a page was accessed, not what page.
-   Same with keyvalue reads, currently chatter is not credibly private because the RPC node can see all queries
-   With PIR the RPC node would not be able to see what chatter inbox you are reading from


There are many types of PIR
-   Single server
-   Multi server schemes that relies on multiple servers handling parts of the query.
    -   This requires 1 out of N servers being honest and not sharing the PIR query information.
    -   TOR is interesting analogue here as it is composed of altruistic operators + 3 nodes for privacy.
    -   TOR seems to work, so maybe a 3 server PIR scheme could work.




The biggest problem with PIR schemes is that each request has to touch the complete dataset, ie if you process a request and do not read from a page then the RPC node knows it is impossible for that request to be interested in that page.

This means if dataset is 1 petabyte, you need to scan the whole dataset and do some operation on each byte.

So every client query requires reading 1 petabyte of data into memory.

Solving this in a decentralized manner is probably one of the most difficult engineering challenges possible.


There is some hope though.
-   Queries can be batched somewhat, you can put some data in CPU cache to maybe batch up to 256 MB of requests per complete memory scan.
-   There is some possible schemes to have have multiple servers share burden of memory scan so each server does not need to scan 1 petabyte of memory, however to maintain same privacy guarantees this requires each RPC node to themselves do PIR queries which leads to overhead, maybe 10x if each "memory shard" has 10 nodes within it.


With current PIR schemes this would require probably 100+ PB/s of network wide memory bandwidth in order to serve 1 PB dataset with reasonable latency and privacy guarantees. Realistically even more, possibly a lot more.

While this is absolutely insane hardware requirements, it is not completely impossible. If you really wanted to, it could be done.

Vastrum's architecture with keyvalue storage was choosen specifically because it is the only type of storage which could work with private information retrieval.



Future
-   LLM's need a lot of memory bandwidth, perhaps this could drive significant hardware improvements and decreases in memory bandwidth cost over time.
