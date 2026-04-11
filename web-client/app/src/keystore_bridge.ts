import { get_private_key, set_private_key } from "../wasm/pkg/vastrum_wasm";

const STORAGE_TIMEOUT_MS = 5000;

export async function initSharedKeystore(): Promise<void> {
  const keystoreUrl = computeKeystoreUrl();
  if (keystoreUrl === null) {
    ensureLocalKey();
    return;
  }

  let iframe: HTMLIFrameElement;
  try {
    iframe = await createIframe(keystoreUrl.href);
  } catch (e) {
    console.warn("[keystore] iframe load failed, falling back to local key", e);
    ensureLocalKey();
    return;
  }

  const targetOrigin = keystoreUrl.origin;

  try {
    const shared = await requestKeystore(iframe, targetOrigin, { type: "keystore:get" });
    if (typeof shared === "string" && shared.length > 0) {
      set_private_key(shared);
      return;
    }

    let local: string;
    try {
      local = get_private_key();
    } catch (e) {
      console.warn("[keystore] failed to read/generate local key", e);
      return;
    }

    const authoritative = await requestKeystore(iframe, targetOrigin, {
      type: "keystore:set_if_empty",
      value: local,
    });
    if (typeof authoritative === "string" && authoritative.length > 0 && authoritative !== local) {
      set_private_key(authoritative);
    }
  } catch (e) {
    console.warn("[keystore] bridge failed, falling back to local key", e);
    ensureLocalKey();
  }
}

function ensureLocalKey() {
  try { get_private_key(); } catch (e) { console.warn("[keystore] local key access failed", e); }
}

function computeKeystoreUrl(): URL | null {
  const { protocol, hostname, port } = window.location;

  if (hostname === "localhost" || hostname === "127.0.0.1") {
    return null;
  }

  const parts = hostname.split(".");
  const portPart = port ? `:${port}` : "";

  if (parts[parts.length - 1] === "localhost") {
    if (parts.length === 1) return null;
    return new URL(`${protocol}//localhost${portPart}/keystore.html`);
  }

  if (parts.length <= 2) return null;

  const root = parts.slice(-2).join(".");
  return new URL(`${protocol}//${root}${portPart}/keystore.html`);
}

function createIframe(src: string): Promise<HTMLIFrameElement> {
  return new Promise((resolve, reject) => {
    const iframe = document.createElement("iframe");
    iframe.setAttribute("aria-hidden", "true");
    iframe.style.display = "none";
    iframe.style.width = "0";
    iframe.style.height = "0";
    iframe.style.border = "0";

    const cleanup = () => {
      iframe.removeEventListener("load", onLoad);
      iframe.removeEventListener("error", onError);
    };
    const onLoad = () => { cleanup(); resolve(iframe); };
    const onError = () => { cleanup(); reject(new Error("keystore iframe load error")); };

    iframe.addEventListener("load", onLoad);
    iframe.addEventListener("error", onError);
    iframe.src = src;
    document.body.appendChild(iframe);
  });
}

type KeystoreRequest =
  | { type: "keystore:get" }
  | { type: "keystore:set_if_empty"; value: string };

function requestKeystore(
  iframe: HTMLIFrameElement,
  targetOrigin: string,
  request: KeystoreRequest
): Promise<string | null> {
  return new Promise((resolve, reject) => {
    const requestId = Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
    const message = { ...request, request_id: requestId };

    const timer = window.setTimeout(() => {
      window.removeEventListener("message", handler);
      reject(new Error("keystore request timeout"));
    }, STORAGE_TIMEOUT_MS);

    function handler(event: MessageEvent) {
      if (event.source !== iframe.contentWindow) return;
      if (event.origin !== targetOrigin) return;
      const data = event.data;
      if (!data || typeof data !== "object") return;
      if ((data as { request_id?: unknown }).request_id !== requestId) return;
      window.clearTimeout(timer);
      window.removeEventListener("message", handler);
      const typed = data as { type: string; value?: string | null; code?: string };
      if (typed.type === "keystore:result") {
        resolve(typed.value ?? null);
      } else if (typed.type === "keystore:error") {
        reject(new Error(`keystore error: ${typed.code ?? "unknown"}`));
      } else {
        reject(new Error("unexpected keystore response"));
      }
    }

    window.addEventListener("message", handler);
    const targetWindow = iframe.contentWindow;
    if (!targetWindow) {
      window.clearTimeout(timer);
      window.removeEventListener("message", handler);
      reject(new Error("keystore iframe has no contentWindow"));
      return;
    }
    targetWindow.postMessage(message, targetOrigin);
  });
}
