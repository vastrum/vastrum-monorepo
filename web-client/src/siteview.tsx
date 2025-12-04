import { useEffect, useRef, useState } from "react";
//import DOMPurify from "dompurify";
import { get_page, make_call } from "../wasm/pkg";
import nojsembedd from './nojsembedd.js?raw';

type PageResponse = {
  content: string;
  site_id: string;
};
type SanitizedPageResponse = {
  content: string;
  site_id: string;
};
interface SiteViewProps {
  page_route: string;
  set_page_route: (page_route: string) => void;
  site_id: string;
  set_site_id: (site_id: string) => void;

}

async function fetch_and_sanitize_page(page_route: string): Promise<SanitizedPageResponse> {

  const response = await get_page(page_route);
  const result: PageResponse = response;
  return { content: result.content, site_id: result.site_id };

  //todo
  //here need to sanitize all external links
  //also need to sanitize javascript
  //Adds a hook to remove all links that could change url to external urls
  //DOMPurify.addHook("afterSanitizeAttributes", (node) => {
  // proxy form actions
  /*if ("action" in node) {
    node.setAttribute("action", ``);
    node.remove();
  }*/
  // proxy regular HTML links
  /*if (node.tagName === "A") {
    if (node.hasAttribute("href")) {
      node.setAttribute("href", ``);
      node.remove();
    }
  }*/
  // proxy SVG/MathML links
  /*if (node.hasAttribute("xlink:href")) {
    node.setAttribute("xlink:href", ``);
    node.remove();
  }*/
  //});
  //const sanitized_page = DOMPurify.sanitize(result.content);
}

export function SiteView({ page_route, set_page_route, site_id, set_site_id }: SiteViewProps) {
  const [pageData, setPageData] = useState("");
  //const [isLoading, setIsLoading] = useState(true);
  const [rerenderhacknonce, setRerenderHackNonce] = useState(1);
  const iframeRef = useRef<HTMLIFrameElement>(null);

  //load javascript for handling buttons into IFrame
  useEffect(() => {
    const iframe = iframeRef.current;
    if (!iframe) return;

    iframe.onload = () => {
      const iframeDoc = iframe.contentDocument || iframe.contentWindow?.document;
      if (iframeDoc) {
        const script = iframeDoc.createElement('script');
        script.textContent = nojsembedd;
        iframeDoc.body.appendChild(script);
      }
    };
  }, []);

  useLinkClickHandlers(set_page_route, setRerenderHackNonce, site_id);

  useEffect(() => {
    (async () => {
      //setIsLoading(true);
      let page_content = await fetch_and_sanitize_page(page_route);
      set_site_id(page_content.site_id);
      setPageData(page_content.content);
      //setIsLoading(false);
    })();
  }, [page_route, rerenderhacknonce]);


  return (
    <>
      <div>
        <iframe
          ref={iframeRef}
          srcDoc={`
            <html>
            <head>
              <base target="_parent">
            </head>
            <body>
            ${pageData}
            </body>
            </html>
          `}

          className="fixed w-full h-full border-0"
        />
      </div>
    </>
  );
}

function useLinkClickHandlers(
  setPageRoute: (route: string) => void,
  setRerenderHackNonce: (nonce: number) => void,
  site_id: string
) {
  // Listen for messages from iframe
  useEffect(() => {
    const handleMessage = (event: MessageEvent) => {
      var data = JSON.parse(event.data);

      var is_navigation = data.msg_type == 0;
      var is_transaction = data.msg_type == 1;
      if (is_navigation) {
        setPageRoute(data.message);
      }

      if (is_transaction) {
        const payload = data.message;
        (async () => {
          await make_call(site_id, payload);
        })();
        setTimeout(() => { setRerenderHackNonce(Math.random()) }, 4000);
      }

    };

    window.addEventListener('message', handleMessage);

    return () => window.removeEventListener('message', handleMessage);
  }, [site_id]);
}
