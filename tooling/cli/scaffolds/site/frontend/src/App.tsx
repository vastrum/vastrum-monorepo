import { useState } from 'react';
import { RouterProvider, Outlet, createMemoryRouter } from 'react-router-dom';
import './styles/index.css';
import { createVastrumReactRouter, await_tx_inclusion } from '@vastrum/react-lib';
import { set_value, get_value } from '../wasm/pkg';

function Layout() {
    return (
        <div className="min-h-screen bg-app-bg text-app-text">
            <div className="max-w-xl mx-auto px-4 py-8">
                <Outlet />
            </div>
        </div>
    );
}

function Home() {
    const [writeKey, setWriteKey] = useState('');
    const [writeValue, setWriteValue] = useState('');
    const [writeStatus, setWriteStatus] = useState('');

    const [readKey, setReadKey] = useState('');
    const [readResult, setReadResult] = useState<string | null>(null);
    const [readLoading, setReadLoading] = useState(false);

    const handleWrite = async () => {
        if (!writeKey || !writeValue) return;
        setWriteStatus('Sending...');
        try {
            const txHash = await set_value(writeKey, writeValue);
            setWriteStatus('Waiting for confirmation...');
            await await_tx_inclusion(txHash);
            setWriteStatus('Saved!');
        } catch (err) {
            setWriteStatus('Error: ' + String(err));
        }
    };

    const handleRead = async () => {
        if (!readKey) return;
        setReadLoading(true);
        try {
            const value = await get_value(readKey);
            setReadResult(value ?? '(not found)');
        } catch (err) {
            setReadResult('Error: ' + String(err));
        } finally {
            setReadLoading(false);
        }
    };

    return (
        <div className="flex flex-col gap-8">
            <h1 className="text-3xl font-bold">{"{{Name}}"}</h1>

            <div className="bg-app-surface border border-app-border rounded-lg p-5 flex flex-col gap-3">
                <h2 className="text-lg font-semibold">Write</h2>
                <input
                    className="bg-app-bg border border-app-border rounded px-3 py-2 text-app-text"
                    placeholder="Key"
                    value={writeKey}
                    onChange={e => setWriteKey(e.target.value)}
                />
                <input
                    className="bg-app-bg border border-app-border rounded px-3 py-2 text-app-text"
                    placeholder="Value"
                    value={writeValue}
                    onChange={e => setWriteValue(e.target.value)}
                />
                <button
                    className="bg-app-accent hover:bg-app-accent-hover text-white rounded px-4 py-2 font-medium transition-colors"
                    onClick={handleWrite}
                >
                    Save
                </button>
                {writeStatus && <p className="text-app-text-secondary text-sm">{writeStatus}</p>}
            </div>

            <div className="bg-app-surface border border-app-border rounded-lg p-5 flex flex-col gap-3">
                <h2 className="text-lg font-semibold">Read</h2>
                <div className="flex gap-2">
                    <input
                        className="flex-1 bg-app-bg border border-app-border rounded px-3 py-2 text-app-text"
                        placeholder="Key"
                        value={readKey}
                        onChange={e => setReadKey(e.target.value)}
                    />
                    <button
                        className="bg-app-accent hover:bg-app-accent-hover text-white rounded px-4 py-2 font-medium transition-colors"
                        onClick={handleRead}
                        disabled={readLoading}
                    >
                        {readLoading ? '...' : 'Read'}
                    </button>
                </div>
                {readResult !== null && (
                    <div className="bg-app-bg border border-app-border rounded px-3 py-2 text-app-text font-mono text-sm min-h-[2.5rem]">
                        {readResult}
                    </div>
                )}
            </div>
        </div>
    );
}

const routes = [
    {
        element: <Layout />,
        children: [
            { path: '/', element: <Home /> },
        ],
    },
];

export const router = await createVastrumReactRouter(routes, createMemoryRouter);

export default function App() {
    return <RouterProvider router={router} />;
}
