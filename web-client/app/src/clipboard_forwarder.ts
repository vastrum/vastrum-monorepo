export const CLIPBOARD_FORWARDER_SCRIPT = `
<script>
(function() {
  var clipboard = {
    writeText: function(text) {
      window.parent.postMessage({ type: 'iframe-clipboard-copy', text: text }, '*');
      return Promise.resolve();
    }
  };
  Object.defineProperty(navigator, 'clipboard', {
    get: function() { return clipboard; },
    configurable: true
  });
})();
</script>
`;

export function handleClipboardCopyMessage(e: MessageEvent) {
  const data = e.data;
  if (data && typeof data === 'object' && data.type === 'iframe-clipboard-copy' && typeof data.text === 'string') {
    navigator.clipboard.writeText(data.text);
  }
}
