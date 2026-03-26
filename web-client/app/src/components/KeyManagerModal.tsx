import { useState } from "react";
import { get_private_key, get_ed25519_public_key, set_private_key } from "../../wasm/pkg/vastrum_wasm";
import KeyControl from "./KeyControl";
import CopyButton from "./CopyButton";

export default function KeyManager() {
    const [isOpen, setIsOpen] = useState(false);

    const [privateKey, setPrivateKey] = useState(get_private_key());
    const [pubKey, setPubKey] = useState(get_ed25519_public_key());

    const [newKey, setNewKey] = useState("");


    const handleImportKey = () => {
        if (newKey.trim() !== "") {
            set_private_key(newKey.trim());
            setPrivateKey(get_private_key());
            setPubKey(get_ed25519_public_key());
            setNewKey("");
        }
    };

    return (
        <div className="z-50 flex w-full fixed pointer-events-none">
            <KeyControl
                pubKey={pubKey}
                setIsOpen={() => setIsOpen(true)}
            />

            {/* Modal */}
            {
                isOpen && (
                    <div
                        className="fixed inset-0 bg-black/70 z-50 flex items-center justify-center pointer-events-auto"
                        onClick={() => setIsOpen(false)}
                    >
                        <div
                            className="bg-gray-800 rounded-lg max-w-md w-full p-6 mx-4"
                            onClick={(e) => e.stopPropagation()}
                        >
                            <h2 className="text-xl font-bold text-white mb-2">Your public key</h2>
                            <p className="text-blue-400 bg-gray-900 rounded px-3 py-2 mb-4 break-all select-all text-sm font-mono">
                                {pubKey}
                            </p>

                            <h2 className="text-xl font-bold text-white mb-2">Your current private key</h2>
                            <p className="text-blue-400 bg-gray-900 rounded px-3 py-2 mb-4 break-all select-all text-sm font-mono">
                                {privateKey}
                            </p>

                            <CopyButton
                                value={privateKey}
                                label="Copy Private Key"
                                className="w-[200px]"
                            />

                            <div className="flex flex-col gap-2 my-4">
                                <input
                                    type="text"
                                    placeholder="Enter new private key"
                                    value={newKey}
                                    onChange={(e) => setNewKey(e.target.value)}
                                    className="w-full px-3 py-2 bg-gray-900 text-white border border-gray-600 rounded text-sm placeholder-gray-500 focus:outline-none focus:border-blue-500"
                                />
                                <button
                                    className="px-4 py-2 text-sm font-medium text-white bg-blue-600 hover:bg-blue-500 rounded"
                                    onClick={handleImportKey}
                                >
                                    Update Private Key
                                </button>
                            </div>

                            <div className="flex justify-end gap-3">
                                <button
                                    className="px-4 py-2 text-sm font-medium text-gray-300 hover:text-white bg-gray-700 hover:bg-gray-600 rounded"
                                    onClick={() => setIsOpen(false)}
                                >
                                    Close
                                </button>
                            </div>
                        </div>
                    </div>
                )
            }
        </div >
    );
}
