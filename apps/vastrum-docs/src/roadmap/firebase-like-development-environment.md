# Firebase-like Development Environment

Currently bootstrapping a project is quite complex.

A full stack project requires
-   ABI crate
-   Deploy crate
-   Frontend react + frontend WASM crate
-   Smart contract crate

It would be better if you just had one project folder to care about.

I think the best way to achieve this is to try to replicate the developer experience of firebase.

Basically you would declare DB schema in a Javascript file, the NPM library would parse this, automatically create a smart contract and create deploy script for it.

The schema could support things such as
-   Access control based on msg.sender
-   Automatic content addressed hashing
-   Automatic pagination and sorted access through KvVecBtree in generated smart contract
-   Typed access of the DB.

The schema could handle things such as
-   Automatic scaling across multiple execution shards




This works well with most CRUD applications which are most of the applications.


With this developing a decentralized website would become very similar to developing a firebase application.