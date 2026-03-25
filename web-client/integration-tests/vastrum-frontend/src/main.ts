import { run_tests } from '../wasm/pkg';

try {
    await run_tests();
    console.log('\nAll tests passed.');
    window.parent.postMessage({ type: 'test-result', status: 'success' }, '*');
} catch (err) {
    console.error('\nTest failed:', err);
    window.parent.postMessage({ type: 'test-result', status: 'failed' }, '*');
}
