# IFrame Sandbox

All websites are executed inside a sandboxed iframe.

This prevents any external network requests.

It also limits what the website can do
-   No pop ups
-   No alerts

Because there is no loading of external resources allowed by the sandbox, all resources needed by the application needs to be inlined.
-   CSS
-   Javascript
-   WASM inlined as base64

Code from web-client siteview.tsx

```html
<iframe
    srcDoc={`
    <html>
    <head>
        <meta http-equiv="Content-Security-Policy"
        content="default-src 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval' data: blob:; connect-src 'none'; form-action 'none';">
        ${CLICK_INTERCEPTOR_SCRIPT}
        ${CONSOLE_FORWARDER_SCRIPT}
        ${CLIPBOARD_FORWARDER_SCRIPT}
        ${TITLE_FORWARDER_SCRIPT}
    </head>
    <body>
    ${pageData}
    </body>
    </html>
    `}
    sandbox="allow-scripts"
    allowFullScreen
    className="w-full flex-1 border-0"
    style={{ minHeight: 0, height: '100%' }}
/>
```