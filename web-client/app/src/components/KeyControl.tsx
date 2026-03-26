import { useState } from "react";
import CopyButton from "./CopyButton";

interface KeyControlProps {
    pubKey: string;
    setIsOpen: () => void;
}

const KeyControl: React.FC<KeyControlProps> = ({ pubKey, setIsOpen }) => {
    const [expanded, setExpanded] = useState(false);

    return (
        <>
            {expanded && (
                <div className={"z-[50] mx-auto inline-flex items-center gap-3 bg-gray-800 border-gray-600 rounded-b-sm shadow-xl p-1 border-b border-l border-r pointer-events-auto"}>

                    <button
                        className="px-3 py-1 rounded text-sm font-medium text-gray-300 hover:text-white bg-gray-700 hover:bg-gray-600 transition-colors"
                        onClick={() => setExpanded(!expanded)}
                        title="Minimize"
                    >
                        -
                    </button>

                    <>
                        <CopyButton
                            value={pubKey}
                            label="Copy Public Key"
                            className="w-[200px] pointer-events-auto"
                        />

                        <button
                            className="px-5 py-2 rounded text-sm font-medium text-white bg-blue-600 hover:bg-blue-500"
                            onClick={setIsOpen}
                        >
                            Settings
                        </button>
                    </>
                </div>)
            }

            {
                !expanded && (
                    <div className={"z-[50] mx-auto inline-flex items-center bg-gray-800 rounded-b-sm shadow-xl pointer-events-auto"}>

                        <button
                            className="text-sm w-7 h-5 rounded-b-sm text-gray-300 hover:text-white bg-gray-700 hover:bg-gray-600 transition-colors"
                            onClick={() => setExpanded(!expanded)}
                            title="Expand"
                        >
                            +
                        </button>
                    </div>
                )
            }
        </>
    );
};

export default KeyControl;
