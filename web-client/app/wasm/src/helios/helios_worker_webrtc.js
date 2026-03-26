//uses webrtc to get rpc data, this is achieved by monkey patching self.fetch and hijacking the request and sending it through webrtc
//instead of regular fetch
//__ORIGIN__ and similar are replaced with actual values by fn build_worker_js()

const origin = '__ORIGIN__';
const execution_rpc = '__EXECUTION_RPC__';
const consensus_rpc = '__CONSENSUS_RPC__';
const checkpoint = '__CHECKPOINT__';
const network = '__NETWORK__';

try {
    const mod = await import(`${origin}/wasm-helios-js-worker-only/vastrum_helios_worker_wasm.js`);
    await mod.default();

    const _pending_fetches = new Map();
    let _next_fetch_id = 1;
    //monkeypatch of self.fetch, takes requests, encodes it and using postmessages sends it to main web-client "thread" from helios-worker
    //web-client then handled webrtc communication
    self.fetch = (input, init) => {
        const req = new Request(input, init);
        const url = req.url;
        return req.arrayBuffer().then(ab => new Promise((resolve, reject) => {
            const id = _next_fetch_id++;
            _pending_fetches.set(id, { resolve, reject, url });
            const body = ab.byteLength > 0 ? Array.from(new Uint8Array(ab)) : null;
            self.postMessage({ type: 'FetchIntercept', id, url, method: req.method, body });
        }));
    };

    //web-client posts responses to hijacked fetch calls through the FetchResponse message
    //however web-client also sends regular rpc requests using this path, which is handled by else branch
    self.onmessage = async (e) => {
        if (e.data.type === 'FetchResponse') {
            const pending_req = _pending_fetches.get(e.data.id);
            if (pending_req) {
                _pending_fetches.delete(e.data.id);
                const r = new Response(new Uint8Array(e.data.body), {
                    status: e.data.status,
                    headers: { 'content-type': e.data.content_type }
                });
                Object.defineProperty(r, 'url', { value: pending_req.url });
                pending_req.resolve(r);
            }
            return;
        } else {
            const { id, request } = e.data;
            self.postMessage({ type: 'Response', id, data: await mod.worker_rpc(request) });
        }
    };

    mod.init_helios(JSON.stringify({ execution_rpc, consensus_rpc, checkpoint, network }));



    self.postMessage({ type: 'Ready' });
} catch (e) {
    self.postMessage({ type: 'Error', error: e.toString() });
}
