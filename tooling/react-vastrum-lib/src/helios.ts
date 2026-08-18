
import { send_eth_rpc_request } from '../wasm/pkg';

export function createHeliosProvider(): VastrumHeliosProvider {
    return new VastrumHeliosProvider();
}

type Request = {
    method: string;
    params: any[];
};

export class VastrumHeliosProvider {

    public constructor() {
    }

    public async request(req: Request): Promise<any> {
        try {
            return await send_eth_rpc_request(req);
        } catch (err: any) {
            throw new Error(err.toString());
        }
    }

    on(
        _eventName: string,
        _handler: (data: any) => void
    ): void {
    }

    removeListener(
        _eventName: string,
        _handler: (data: any) => void
    ): void {
    }
}
