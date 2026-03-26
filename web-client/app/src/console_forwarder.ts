export const CONSOLE_FORWARDER_SCRIPT = `
<script>
(function() {
  var origLog = console.log;
  var origError = console.error;
  var origWarn = console.warn;
  function forward(type, origFn, args) {
    origFn.apply(console, args);
    try {
      var parts = [];
      for (var i = 0; i < args.length; i++) {
        try {
          parts.push(typeof args[i] === 'string' ? args[i] : JSON.stringify(args[i]));
        } catch(e) {
          parts.push(String(args[i]));
        }
      }
      window.parent.postMessage({ type: type, message: parts.join(' ') }, '*');
    } catch(e) {}
  }
  console.log = function() { forward('iframe-log', origLog, arguments); };
  console.error = function() { forward('iframe-error', origError, arguments); };
  console.warn = function() { forward('iframe-warn', origWarn, arguments); };
})();
</script>
`;

export function handleConsoleForwardMessage(e: MessageEvent) {
  const data = e.data;
  if (data && typeof data === 'object') {
    if (data.type === 'iframe-log') {
      console.log('[SITE_LOG]', data.message);
    } else if (data.type === 'iframe-error') {
      console.error('[SITE_LOG]', data.message);
    } else if (data.type === 'iframe-warn') {
      console.warn('[SITE_LOG]', data.message);
    }
  }
}
