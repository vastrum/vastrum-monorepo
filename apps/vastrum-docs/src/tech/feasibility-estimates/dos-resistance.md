# DOS Resistance

Without gas, there is a huge problem of how to deal with spam at the protocol layer.

The only feasible way i have found is to use proof of work burnt electricity as a kind of universal value transfer.

I have not yet fully developed the approach to this and there are still many open questions.

Below i have tried to model it, probably not best way to model it but gives some understanding of dynamics of the problem.


## Model

Scaling is basically built on assumption that blockchain layer throughput need for genuine users is already satisfied, 
only additional property needed is graceful failure on DOS attacks, 
ie as load on network increases, increase the required proof of work to get your transaction included 
to maintain service even if somewhat degraded.

## Blockchain resources

Assume block throughput is 5 MB/s

Because execution could plausibly execute 20k TPS, assume blockchain throughput is only bandwidth limited and not compute limited.


## Burnable POW value from users

What are some plausible numbers of value that a user could burn?

These numbers are heavily estimated and probably less in reality considering CPU usage restrictions of web browser environment and other unknowns, however gives a hint of potential feasibility.

    Assume phone can burn 5w 
        5wh / h
        0.05 USD / kwh 
        0,00025 USD / hour

    Assume laptop/desktop can burn 50w
        0,0025 USD / hour

    For margin of safety 

        phone 0,0001 USD / hour
        laptop/desktop = 0.001 USD / hour

| Device | Power | Mined Value |
|--------|-------|---------------|
| Phone | 5w | 0.0001 USD / h |
| Laptop/desktop | 50w | 0.001 USD / h |


## Simple model of a DOS

If DOSer spends 10 USD per hour, then to access half of blockspace genuine users also needs to burn 10 USD per hour.



    5 MB / s * 3600 = 18000 MB per hour of blockspace

    18000 MB per hour of blockspace available per execution shard

    9000 MB to DOSer

    9000 MB to actual users

### if DOSer spends 10 USD per hour

    9000 MB / 10 USD = 900 MB / 1 USD

    1 USD of burnt POW buys 900 MB of blockspace 

    Phone users can burn 0,0001 USD per hour

    0,0001 * 900 MB = 90 KB
    Phone users mine 90 KB per hour
    3,6 seconds to mine 100b transaction


    Laptop/desktop users can burn 0,001 USD per hour

    0,001 * 900 MB = 900 KB
    Laptop/desktop users mine 900 KB per hour
    0,36 seconds to mine 100b transaction

### if DOSer spends 100 USD per hour > 

    1 USD of burnt POW buys 90 MB

    0,0001 * 90 MB = 9 KB
    Phone users mine 9 KB per hour
    36 seconds to mine 100 byte transaction

    0,001 * 90 MB = 90 KB
    Laptop/desktop users mine 90 KB per hour
    3,6 seconds to mine 100 byte transaction


### if DOSer spends 1k USD per hour > 

    1 USD of burnt POW buys 9 MB

    0,0001 * 9 MB = 1 KB
    Phone users mine 1 KB per hour
    360 seconds to mine 100 byte transaction

    0,001 * 9 MB = 9 KB
    Laptop/desktop users mine 9 KB per hour
    36 seconds to mine 100 byte transaction

| DOS spend | Phone mining time (100 byte tx) | Desktop mining time (100 byte tx) |
|-----------|----------------------------|-------------------------------|
| 10 USD/h  | 3.6s                       | 0.36s                         |
| 100 USD/h | 36s                        | 3.6s                          |
| 1k USD/h  | 360s                       | 36s                           |


#### Summary

Above 50-100 USD / h DOS level user experience will probably start to get severely impacted.

In general it is very cheap to DOS the network, but it is not free. Especially for sustained campaigns. 

The network has some level of credible resistance to DOSing, while performance is degraded some level of user throughput is maintained.


This analysis is probably faulty in some manner and it is also very handwavy, 
but it does not seem impossible for POW gas to somewhat viable and DOS resistant.


In regular happy path which will happen most of the time there will be no DOS, in that case can just assume all of the blockspace will be dedicated to productive usage.

As long as there is credible resistance to DOSing most DOS attacks will not happen.

#### Write only writes
With write only writes can assume "blob" uploads are basically DOS resistant as they can be executed inside any execution shard. The remaining DOSable parts are the stateful writes which has to be executed inside a single execution shard, ie updating the head_commit for Gitter which is basically < 200 byte transaction. 

So hopefully you can assume most of the transactions inside the 5 MB/s of execution shard blockspace are small transactions which increases DOS resistance as execution shard TPS can be very high.



### Further improvements to DOS resistance
-   More execution shards, this DOS resistance level is per execution shard
-   Application level spam protection, ie the application can reject transactions at mempool level
-   Token gas payments, this will probably be needed anyway if you need to upload 10+ MB
-   Some kind of ZK proof of history, ie prove you have had an active account for some time that has had made many POW transactions pre DOS attack, a kind of proof of reputation




### Misc ideas

#### Sponsored transactions
Pay for a Concord server to have guaranteed TPS access using gas tokens, then kick any spammers who uses up capacity in that server.


#### Split application across several execution shards
The general execution shard concept is to have each site be hosted on its own execution shard. However maybe for example you could also split each Gitter repo into its own execution shard also?

Basically it would look like this.

-   Each Gitter repository is hosted on its own execution shards which has maybe 20k TPS of capacity.

-   So instead of being able to DOS Gitter with 50 USD / hour you would need to spend 50 USD / hour for each Gitter repository you are interested in DOSing.

-   This is definitively not needed for scaling, but the credible resistance to DOSing increases significantly.

-   In the normal state the whole of Gitter would just live inside 1 execution shard, there would be some kind of dynamic allocation of execution shards as needed to scale up when under DOS attack.




## Current implementation

-   This is not implemented in current version of Vastrum.
-   Currently uses SHA256 which is not a very good POW algorithm for this use case, would need to pick another ASIC resistant algorithm. 
-   Currently all POW hashes are accepted and there is no gas implemented to limit compute usage by each transaction.