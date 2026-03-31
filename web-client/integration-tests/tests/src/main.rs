#[tokio::main]
async fn main() {}

#[cfg(test)]
mod tests {
    use headless_chrome::{Browser, LaunchOptions, Tab};
    use vastrum_native_lib::deployers::build::run;
    use serial_test::serial;
    use vastrum_shared_types::ports::HTTP_RPC_PORT;
    use std::time::{Duration, Instant};
    use web_client_integration_tests_abi::ContractAbiClient;

    fn ensure_network() {
        vastrum_native_lib::test_support::ensure_localnet("../contract", "../contract/out");
    }

    async fn deploy_site(frontend_dir: &str) -> String {
        let total = Instant::now();
        ensure_network();
        eprintln!(
            "[bench] ensure_network (from deploy_site): {:.1}s",
            total.elapsed().as_secs_f64()
        );

        let t = Instant::now();
        eprintln!("[bench] building frontend: {frontend_dir} (npm run build)...");
        run("npm run build", frontend_dir);
        eprintln!("[bench] frontend build ({frontend_dir}): {:.1}s", t.elapsed().as_secs_f64());

        let t = Instant::now();
        let html = std::fs::read_to_string(format!("{}/dist/index.html", frontend_dir))
            .expect("Failed to read built HTML");
        eprintln!(
            "[bench] read HTML ({} bytes): {:.1}ms",
            html.len(),
            t.elapsed().as_secs_f64() * 1000.0
        );

        let t = Instant::now();
        eprintln!("[bench] deploying contract...");
        let brotli = vastrum_shared_types::compression::brotli::brotli_compress_html(&html);

        let client = ContractAbiClient::deploy("../contract/out/contract.wasm", brotli).await;
        eprintln!("[bench] contract deploy: {:.1}s", t.elapsed().as_secs_f64());

        let site_id = client.site_id().to_string();
        let url = format!("http://{site_id}.localhost:{HTTP_RPC_PORT}");
        eprintln!(
            "[bench] deploy_site({frontend_dir}) total: {:.1}s",
            total.elapsed().as_secs_f64()
        );
        url
    }

    fn install_test_result_listener(tab: &Tab) {
        tab.call_method(headless_chrome::protocol::cdp::Page::AddScriptToEvaluateOnNewDocument {
            source: r#"
                    window.__testResult = { status: 'pending' };
                    window.__consoleLogs = [];
                    window.__consoleErrors = [];
                    window.__logIndex = 0;
                    window.__errIndex = 0;
                    const origLog = console.log;
                    const origErr = console.error;
                    const origWarn = console.warn;
                    console.log = function(...args) { window.__consoleLogs.push(args.map(String).join(' ')); origLog.apply(console, args); };
                    console.error = function(...args) { window.__consoleErrors.push(args.map(String).join(' ')); origErr.apply(console, args); };
                    console.warn = function(...args) { window.__consoleErrors.push('WARN: ' + args.map(String).join(' ')); origWarn.apply(console, args); };
                    window.addEventListener('unhandledrejection', (event) => {
                        window.__consoleErrors.push('UNHANDLED REJECTION: ' + String(event.reason));
                    });
                    window.addEventListener('error', (event) => {
                        window.__consoleErrors.push('UNCAUGHT: ' + event.message + ' at ' + event.filename + ':' + event.lineno);
                    });
                    window.addEventListener('message', (event) => {
                        if (event.data && event.data.type === 'test-result') {
                            window.__testResult.status = event.data.status;
                        }
                        if (event.data && event.data.type === 'iframe-log') {
                            window.__consoleLogs.push('[iframe] ' + event.data.message);
                        }
                        if (event.data && event.data.type === 'iframe-error') {
                            window.__consoleErrors.push('[iframe] ' + event.data.message);
                        }
                    });
                "#
            .to_string(),
            world_name: None,
            include_command_line_api: None,
            run_immediately: None,
        })
        .expect("Failed to install test result listener");
    }

    /// Dump only NEW console logs/errors since last call.
    fn dump_new_logs(tab: &Tab, elapsed: f64) {
        // Get new console logs
        if let Ok(result) = tab.evaluate(
            r#"(() => {
                const i = window.__logIndex || 0;
                const logs = (window.__consoleLogs || []).slice(i);
                window.__logIndex = (window.__consoleLogs || []).length;
                return logs.join('\n');
            })()"#,
            false,
        ) {
            if let Some(s) = result.value.as_ref().and_then(|v| v.as_str()) {
                if !s.is_empty() {
                    eprintln!("[{elapsed:.1}s console] {s}");
                }
            }
        }
        // Get new console errors
        if let Ok(result) = tab.evaluate(
            r#"(() => {
                const i = window.__errIndex || 0;
                const errs = (window.__consoleErrors || []).slice(i);
                window.__errIndex = (window.__consoleErrors || []).length;
                return errs.join('\n');
            })()"#,
            false,
        ) {
            if let Some(s) = result.value.as_ref().and_then(|v| v.as_str()) {
                if !s.is_empty() {
                    eprintln!("[{elapsed:.1}s ERRORS] {s}");
                }
            }
        }
    }

    fn await_test_result(tab: &Tab) {
        let start = Instant::now();
        let poll_interval = Duration::from_millis(500);
        let mut polls = 0;
        loop {
            let elapsed = start.elapsed().as_secs_f64();
            let result = tab
                .evaluate(r#"window.__testResult.status"#, false)
                .expect("Failed to read test status");

            let status = result.value.as_ref().and_then(|v| v.as_str());
            match status {
                Some("success") => {
                    dump_new_logs(tab, elapsed);
                    eprintln!("[{elapsed:.1}s] TEST PASSED");
                    return;
                }
                Some("failed") => {
                    dump_new_logs(tab, elapsed);
                    panic!("[{elapsed:.1}s] Frontend tests failed");
                }
                _ => {
                    polls += 1;
                    // Dump every 2 seconds (every 4th poll)
                    if polls % 4 == 1 {
                        dump_new_logs(tab, elapsed);
                        // Dump DOM state for diagnostics
                        if let Ok(state) = tab.evaluate(
                            r#"(() => {
                                const iframe = document.querySelector('iframe');
                                const loading = document.querySelector('.animate-spin');
                                const root = document.getElementById('root');
                                return JSON.stringify({
                                    hasIframe: !!iframe,
                                    hasSpinner: !!loading,
                                    rootChildren: root ? root.children.length : -1,
                                    rootText: root ? root.innerText.substring(0, 200) : 'no root',
                                    url: window.location.href,
                                });
                            })()"#,
                            false,
                        ) {
                            if let Some(s) = state.value.as_ref().and_then(|v| v.as_str()) {
                                eprintln!("[{elapsed:.1}s DOM] {s}");
                            }
                        }
                    }
                    std::thread::sleep(poll_interval);
                }
            }
        }
    }

    async fn run_browser_test(url: &str, timeout_secs: u64) {
        let total = Instant::now();
        let url = url.to_string();
        eprintln!("[bench] launching browser for {url} (timeout={timeout_secs}s)...");
        let result = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            tokio::task::spawn_blocking(move || {
                let t = Instant::now();
                let browser = Browser::new(
                    LaunchOptions::default_builder()
                        .idle_browser_timeout(Duration::from_secs(300))
                        .build()
                        .unwrap(),
                )
                .expect("Failed to launch browser");
                let tab = browser.new_tab().expect("Failed to create tab");
                eprintln!("[bench] browser launch: {:.1}s", t.elapsed().as_secs_f64());

                install_test_result_listener(&tab);

                let t = Instant::now();
                eprintln!("[bench] navigating to {url}...");
                tab.navigate_to(&url).expect("Failed to navigate");
                tab.wait_until_navigated().expect("Failed to wait for navigation");
                eprintln!("[bench] navigation: {:.1}s", t.elapsed().as_secs_f64());

                let t = Instant::now();
                eprintln!("[bench] polling for test result...");
                await_test_result(&tab);
                eprintln!("[bench] test execution: {:.1}s", t.elapsed().as_secs_f64());
            }),
        )
        .await;

        eprintln!("[bench] run_browser_test total: {:.1}s", total.elapsed().as_secs_f64());
        match result {
            Ok(join_result) => join_result.expect("Browser task panicked"),
            Err(_) => panic!("Test timed out after {}s", timeout_secs),
        }
    }

    #[tokio::test]
    #[serial]
    #[ignore]
    async fn test_helios_eth_rpc() {
        let total = Instant::now();
        let url = deploy_site("../helios-frontend").await;
        run_browser_test(&url, 180).await;
        eprintln!(
            "\n[bench] === test_helios_eth_rpc TOTAL: {:.1}s ===\n",
            total.elapsed().as_secs_f64()
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_vastrum_iframe_rpc() {
        let total = Instant::now();
        let url = deploy_site("../vastrum-frontend").await;
        run_browser_test(&url, 20).await;
        eprintln!(
            "\n[bench] === test_vastrum_iframe_rpc TOTAL: {:.1}s ===\n",
            total.elapsed().as_secs_f64()
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_starknet_rpc() {
        let total = Instant::now();
        let url = deploy_site("../starknet-frontend").await;
        run_browser_test(&url, 60).await;
        eprintln!(
            "\n[bench] === test_starknet_rpc TOTAL: {:.1}s ===\n",
            total.elapsed().as_secs_f64()
        );
    }
}
