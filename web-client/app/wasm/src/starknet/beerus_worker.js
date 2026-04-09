const origin = '__ORIGIN__';

try {
    const mod = await import(`${origin}/wasm-beerus-worker/vastrum_beerus_worker_wasm.js`);
    await mod.default();
    mod.init_beerus(JSON.stringify({}));
    self.onmessage = async (e) => {
        const { id, request } = e.data;
        self.postMessage({ type: 'Response', id, data: await mod.worker_rpc(request) });
    };
    self.postMessage({ type: 'Ready' });
} catch (e) {
    self.postMessage({ type: 'Error', error: e.toString() });
}
