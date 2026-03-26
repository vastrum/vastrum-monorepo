# Highest Current Technical Risks


There is still a lot of technical risks remaining.

-   Execution sharding
    -   Probably not so difficult as sites are already isolated from each other, but it is a major redesign
    -   How do you determine where sites live? How do you transfer sites + state between execution shards?
-   IFrame sandbox not actually being very secure
    -   This might become a major issue with untrusted websites, especially with user private key held inside web-client
-   Write only data sharding
    -   How to solve data availability?
    -   How to solve RPC read access for users?
-   Sustainable POW
    -   This might be very hard to design the economics for, especially if professional miners starts mining it.
-   Sustainable RPC providers
    -   If nobody can read the data it is basically useless. As the protocol requires many operators it is hard to say if it will be possible to attract enough operators.

-   Private information retrieval (PIR)
    -   Basically impossible

    
    
