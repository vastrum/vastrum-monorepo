//uses http fetch to get data
//__ORIGIN__ and similar are replaced with actual values by fn build_worker_js()

const origin = '__ORIGIN__';
const execution_rpc = '__EXECUTION_RPC__';
const consensus_rpc = '__CONSENSUS_RPC__';
const checkpoint = '__CHECKPOINT__';
const network = '__NETWORK__';

try {
    const mod = await import(`${origin}/wasm-helios-js-worker-only/vastrum_helios_worker_wasm.js`);
    await mod.default();
    mod.init_helios(JSON.stringify({ execution_rpc, consensus_rpc, checkpoint, network }));
    self.onmessage = async (e) => {
        const { id, request } = e.data;
        self.postMessage({ type: 'Response', id, data: await mod.worker_rpc(request) });
    };
    self.postMessage({ type: 'Ready' });
} catch (e) {
    self.postMessage({ type: 'Error', error: e.toString() });
}
