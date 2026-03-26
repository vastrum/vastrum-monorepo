# Threshold TLS

The biggest problem with the current web-client solution is the centralized domain name risk.

Anybody who controls the domain name vastrum.net can disrupt access to the network.


Using a threshold TLS solution, the TLS key could be distributed across multiple entities which collaborate to serve the frontend.

This works by
-   Find a supported TLS crypto scheme which supports threshold operations
-   Generate TLS key
-   Distribute TLS key across multiple servers

When a client connects
-   Round robin DNS connection to any of the servers
-   Send HTTPS handshake request
-   The responding server communicates with the other servers to execute the handshake request.


Theoretical properties
-   No server can control the frontend or return arbitrary data
-   This reduces the frontend centralization risk somewhat


Major problem is that anybody who controls the domain name can just generate a new TLS key, so this does not actually reduce frontend centralization risk at all.

One possible usecase could be
-   User want to host their website on Vastrum
-   Wants to have own domain name, not website.vastrum.net but their web2 domain website.com
-   User generates TLS key
-   Distributes TLS key shards among the Vastrum TLS cluster
-   User points their domain name to the Vastrum name server.
-   The Vastrum TLS cluster can only serve the page the user specified because of threshold operations
-   To serve phishing website or different website perhaps 5 out of 20 TLS operators would have to be malicious and collaborate.


This is a very interesting setup, it would potentially mean you could have the exact same web2 UX including domain name structure while having all of the hosting be decentralized.


There are many technical questions of how this would be implemented, i am not sure if it is even possible.
-   Could at max have 10-20 node operators
-   Each operator would need to have multiple geographically distributed servers in order to minimize latency
-   Because of this probably would need to have professional operators ie known organisations

Of course the best solution would be to develop a native browser that is secure from frontend centralization risks by default, or somehow achieve native integration with current browsers.