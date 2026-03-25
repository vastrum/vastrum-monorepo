//import { EventEmitter } from "eventemitter3";

import { send_eth_rpc_request } from '../wasm/pkg';

export function createHeliosProvider(): VastrumHeliosProvider {
    return new VastrumHeliosProvider();
}

type Request = {
    method: string;
    params: any[];
};

export class VastrumHeliosProvider {
    //eventEmitter;

    public constructor() {
        //this.eventEmitter = new EventEmitter();
    }

    //eip 1193 request()
    public async request(req: Request): Promise<any> {
        try {
            return await send_eth_rpc_request(req);
        } catch (err: any) {
            throw new Error(err.toString());
        }
    }


    //eip 1193 on()
    on(
        _eventName: string,
        _handler: (data: any) => void
    ): void {
        //WEBSOCKETS TODO
        //this.#eventEmitter.on(eventName, handler);
    }

    //eip 1193 removeListener()
    removeListener(
        _eventName: string,
        _handler: (data: any) => void
    ): void {
        //WEBSOCKETS TODO
        //this.#eventEmitter.off(eventName, handler);
    }
}