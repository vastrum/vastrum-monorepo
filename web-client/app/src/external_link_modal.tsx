import { useEffect, useState } from "react";

export const CLICK_INTERCEPTOR_SCRIPT = `
<script>
document.addEventListener('click', function(e) {
  var anchor = e.target.closest ? e.target.closest('a') : null;
  if (!anchor) return;
  var href = anchor.getAttribute('href');
  if (!href) return;
  if (href.startsWith('https://')) {
    e.preventDefault();
    e.stopPropagation();
    var requestId = Date.now();
    var msg = JSON.stringify({
      request_id: requestId,
      method: 'OpenExternalUrl',
      params: JSON.stringify({ url: href })
    });
    window.parent.postMessage(msg, '*');
  }
}, true);
</script>
`;

export function ExternalLinkModal() {
  const [pendingUrl, setPendingUrl] = useState<string | null>(null);

  useEffect(() => {
    const handler = (e: Event) => {
      const url = (e as CustomEvent).detail as string;
      if (typeof url === "string") {
        setPendingUrl(url);
      }
    };
    window.addEventListener("vastrum_open_external_url", handler);
    return () => window.removeEventListener("vastrum_open_external_url", handler);
  }, []);

  if (!pendingUrl) return null;

  return (
    <div
      className="fixed inset-0 bg-black/70 z-50 flex items-center justify-center"
      onClick={() => setPendingUrl(null)}
    >
      <div
        className="bg-gray-800 rounded-lg max-w-md w-full p-6 mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        <h2 className="text-xl font-bold text-white mb-2">This site wants to send you to this external link</h2>
        <p className="text-blue-400 bg-gray-900 rounded px-3 py-2 mb-4 break-all select-all text-sm">
          {pendingUrl}
        </p>
        <p className="text-gray-400 text-sm mb-6">
          Make sure you trust this destination before continuing.
        </p>
        <div className="flex justify-end gap-3">
          <button
            onClick={() => setPendingUrl(null)}
            className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white bg-gray-700 hover:bg-gray-600 rounded"
          >
            Back
          </button>
          <button
            onClick={() => { window.location.href = pendingUrl; }}
            className="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-500 rounded"
          >
            Visit Site
          </button>
        </div>
      </div>
    </div>
  );
}
