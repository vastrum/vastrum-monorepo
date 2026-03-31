import { send_starknet_rpc_request } from '../wasm/pkg';

export async function starknetRpc(rpcUrl: string, method: string, params: any): Promise<any> {
    try {
        return await send_starknet_rpc_request({ rpc_url: rpcUrl, method, params });
    } catch (err: any) {
        throw new Error(err.toString());
    }
}
