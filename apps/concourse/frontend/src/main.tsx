import ReactDOM from 'react-dom/client';
import { StrictMode } from 'react';
import App from './App';

// Suppress React DevTools message in development
if (import.meta.env.DEV) {
    const originalLog = console.log;
    console.log = (...args) => {
        if (typeof args[0] === 'string' && args[0].includes('React DevTools')) {
            return;
        }
        originalLog.apply(console, args);
    };
}

ReactDOM.createRoot(document.getElementById('root')!).render(
    <StrictMode>
        <App />
    </StrictMode>,
);
