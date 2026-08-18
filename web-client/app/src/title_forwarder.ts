const MAX_TITLE_LENGTH = 200;

export const TITLE_FORWARDER_SCRIPT = `
<script>
(function() {
  var descriptor = Object.getOwnPropertyDescriptor(Document.prototype, 'title');
  if (!descriptor || !descriptor.set) { return; }
  function forward(value) {
    try {
      window.parent.postMessage({ type: 'iframe-title-set', title: String(value) }, '*');
    } catch(e) {}
  }
  Object.defineProperty(document, 'title', {
    get: function() { return descriptor.get.call(document); },
    set: function(value) {
      descriptor.set.call(document, value);
      forward(value);
    },
    configurable: true
  });
  // A site whose markup carries a static <title> never runs the setter, so report whatever the
  // document already has once the injected script executes.
  if (document.title) { forward(document.title); }
})();
</script>
`;

export function handleTitleSetMessage(e: MessageEvent) {
  const data = e.data;
  if (!data || typeof data !== "object") return;
  if (data.type !== "iframe-title-set" || typeof data.title !== "string") return;

  const title = data.title.trim().slice(0, MAX_TITLE_LENGTH);
  if (title.length === 0) return;
  document.title = title;
}
