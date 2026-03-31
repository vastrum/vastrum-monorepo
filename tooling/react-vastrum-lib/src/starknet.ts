import { send_starknet_rpc_request } from '../wasm/pkg';

export async function starknetRpc(method: string, params: any): Promise<any> {
    try {
        return await send_starknet_rpc_request({ method, params });
    } catch (err: any) {
        throw new Error(err.toString());
    }
}
