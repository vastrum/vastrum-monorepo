# WebRTC-Direct

The web-client uses WebRTC to connect directly to RPC nodes.

Normally in order to connect to a RPC node from a web browser context you would need a domain name + HTTPS TLS certificate, you cannot directly connect to an IP address.

WebRTC allows for direct connections to an IP address without requiring the server to have a domain name and TLS certificate.

LibP2P solved WebRTC direct from browser-server. WebRTC direct for Vastrum is based on the libP2P implementation.

- [libp2p Browser Connectivity Guide](https://libp2p.io/guides/browser-connectivity/#webrtc)
- [rust-libp2p Browser WebRTC](https://libp2p.io/blog/rust-libp2p-browser-webrtc/)
- [rust-libp2p browser-webrtc example](https://github.com/libp2p/rust-libp2p/tree/master/examples/browser-webrtc)
- [WebRTC-Direct spec](https://github.com/libp2p/specs/blob/master/webrtc/webrtc-direct.md)




Webtransport is a much better alternative to WebRTC with much lower latency, however it is not yet supported by all browsers. Safari recently implemented it and it will hopefully have wide adoption soon.

