# Chaos Engineering

Switch to pipelined consensus.

Built according to chaos engineering ethos
-   Adaptively select block size based on how many previous slots were missed, constantly try to push network to failure.
-   Try to target 100% execution utilization by increasing gas limit until nodes start falling behind and stop voting on execution results

This would work kind of like TCP, you never know what the underlying capacity of the network link is. You just push an increasing amount of packets until the link starts to fail and packets start to get lost.

This should allow 100% utilization of the theoretical max compute limit of the network.

Protocol would be much more robust to failure, in cases of severe network congestion block sizes would fall until block propagation works and consensus starts again.

The protocol will constantly be failing and exposed to much stress, this should make the protocol extremely robust if it can survive it.

----

*This will probably lead to great centralization + rapid increases in hardware requirements as slow validators fall behind, maybe could cap gas limit and block size to prevent this.*
*But then you are back to normal block size target architecture, so maybe not so useful.*
*But i found it interesting*