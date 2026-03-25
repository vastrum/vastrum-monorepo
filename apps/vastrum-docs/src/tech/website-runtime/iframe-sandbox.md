# IFrame Sandbox

All websites are executed inside a sandboxed iframe.

This should prevent any external network requests.

It also limits what the website can do
-   No pop ups
-   No alerts

Because there is no loading of external resources allowed by the sandbox, all resources needed by the application needs to be inlined.
-   CSS
-   Javascript
-   WASM inlined as base64

The iframe sandbox does not protect very well against fingerprinting though, for example it can read what GPU is used by your device.

Hopefully the iframe sandbox will be enough to stop most malicious activities.