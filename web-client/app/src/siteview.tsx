import { useCallback, useEffect, useState } from "react";
import { get_page } from "../wasm/pkg/vastrum_wasm";
import { CONSOLE_FORWARDER_SCRIPT, handleConsoleForwardMessage } from "./console_forwarder";
import { CLICK_INTERCEPTOR_SCRIPT, ExternalLinkModal } from "./external_link_modal";
import { CLIPBOARD_FORWARDER_SCRIPT, handleClipboardCopyMessage } from "./clipboard_forwarder";

function getSubdomain() {
  const hostname = window.location.hostname;
  const parts = hostname.split('.');

  if (hostname === 'localhost' || hostname === '127.0.0.1') {
    return null;
  }
  if (parts[parts.length - 1] === 'localhost') {
    return parts.length > 1 ? parts[0] : null;
  }

  if (parts[0] === 'www') {
    parts.shift();
  }

  if (parts.length <= 2) {
    return null;
  }

  return parts[0];
}

export function SiteView({ page_route }: { page_route: string }) {
  const [pageData, setPageData] = useState("");
  const [isLoading, setIsLoading] = useState(true);

  const fetchPage = useCallback(async () => {
    try {
      const subdomain = getSubdomain();
      //if not a valid subdomain, redirect to docs
      if (subdomain === null) {
        const baseHost = window.location.host.replace(/^www\./, '');
        window.location.href = `${window.location.protocol}//docs.${baseHost}${window.location.pathname}`;
        return;
      }
      const response = await get_page(page_route, subdomain);

      if (response.site_id === "" || response.content === "") {
        setTimeout(fetchPage, 50);
      } else {
        setPageData(response.content);
        setIsLoading(false);
      }
    } catch (e) {
      setTimeout(fetchPage, 50);
    }
  }, [page_route]);

  useEffect(() => {
    fetchPage();
  }, [fetchPage]);

  useEffect(() => {
    window.addEventListener('message', handleConsoleForwardMessage);
    window.addEventListener('message', handleClipboardCopyMessage);
    return () => {
      window.removeEventListener('message', handleConsoleForwardMessage);
      window.removeEventListener('message', handleClipboardCopyMessage);
    };
  }, []);

  return (
    <div className="flex-1 flex flex-col">
      <ExternalLinkModal />
      {isLoading ? (
        <div className="z-1 w-full flex-1 flex flex-col items-center justify-center text-gray-600 px-6 text-center">
          <div className="w-8 h-8 border-4 border-gray-300 border-t-gray-600 rounded-full animate-spin mb-4"></div>
          <div className="text-sm sm:text-base mb-2">Connecting to RPC, polling.. (This might take 5-20 seconds)</div>
          <div className="text-xs sm:text-sm text-gray-400">WebRTC is required to connect to the RPC node, if it is disabled the RPC node connection will fail</div>
        </div>
      ) : (
        <iframe
          srcDoc={`
            <html>
            <head>
              <meta http-equiv="Content-Security-Policy"
                content="default-src 'unsafe-inline' 'unsafe-eval' 'wasm-unsafe-eval' data: blob:; connect-src 'none'; form-action 'none';">
              ${CLICK_INTERCEPTOR_SCRIPT}
              ${CONSOLE_FORWARDER_SCRIPT}
              ${CLIPBOARD_FORWARDER_SCRIPT}
            </head>
            <body>
            ${pageData}
            </body>
            </html>
          `}
          //TODO, allowFullscreen to make docs videos work for now, need to remove this in future
          //as very easy for untrusted code to hijack fullscreen and phish user 
          sandbox="allow-scripts"
          allowFullScreen
          className="w-full flex-1 border-0"
          style={{ minHeight: 0, height: '100%' }}
        />
      )}
    </div>
  );
}
