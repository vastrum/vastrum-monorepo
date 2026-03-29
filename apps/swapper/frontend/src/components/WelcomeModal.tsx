import { Modal } from './Modal';

interface WelcomeModalProps {
    isOpen: boolean;
    onClose: () => void;
}

export function WelcomeModal({ isOpen, onClose }: WelcomeModalProps) {
    return (
        <Modal isOpen={isOpen} onClose={onClose} title="Swapper">
            <div className="p-6 space-y-6">
                <div>
                    <p className="text-app-text-secondary">
                        Swapper is an experimental decentralized Uniswap V2 frontend hosted on Vastrum.
                    </p>

                    <p className="text-app-text-secondary mt-3">Currently you can:</p>
                    <ul className="list-disc list-inside text-app-text-secondary ml-2 space-y-1">
                        <li>Get pair prices by reading pool data from Ethereum</li>
                        <li>Signing transactions is not yet supported</li>
                    </ul>

                    <p className="text-app-text-secondary mt-3">
                        All RPC requests are executed using a Helios light client embedded in your browser. This means all your RPC queries are trustless and verified against Ethereum consensus.
                    </p>
                    <p className="text-app-text-secondary mt-3">
                        Because Helios cryptographically verifies the RPC queries, it can take 5-20 seconds to execute a query.
                    </p>

                    <p className="text-app-text-secondary mt-3">
                        For more details check the Swapper docs.
                    </p>
                </div>

                <div className="flex flex-col gap-1 text-sm">
                    {/* docs */}
                    <a href="https://xpkeuoccopibhnakya3luhrsphalhnqo2ifmxe65murdjft54n3q.vastrum.net/apps/swapper" target="_blank" rel="noopener noreferrer" className="text-app-accent hover:underline">
                        Swapper docs
                    </a>
                    {/* docs */}
                    <a href="https://xpkeuoccopibhnakya3luhrsphalhnqo2ifmxe65murdjft54n3q.vastrum.net/" target="_blank" rel="noopener noreferrer" className="text-app-accent hover:underline">
                        Vastrum docs
                    </a>
                </div>

                {/* Close Button */}
                <div className="flex justify-end pt-2">
                    <button onClick={onClose} className="bg-app-accent-green text-white px-4 py-2 rounded-md font-medium hover:bg-[#2ea043] transition-colors">
                        Close
                    </button>
                </div>
            </div>
        </Modal>
    );
}
