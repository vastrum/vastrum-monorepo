
run_all_tests_slow :; cargo test -- --include-ignored && make run_madsim_tests

run_madsim_tests :; RUSTFLAGS="--cfg madsim --cap-lints allow" RUST_BACKTRACE=1 cargo test -p vastrum-node --features madsim_compliant --test sim_consensus -- --nocapture

fuzz_consensus_madsim:
	for i in $$(seq 1 100); do echo "=== Run $$i/100 ==="; make run_madsim_tests; if [ $$? -ne 0 ]; then echo "FAILED on run $$i"; break; fi; done

check_madsim_determinism :; RUSTFLAGS="--cfg madsim" MADSIM_TEST_CHECK_DETERMINISM=1 cargo test -p vastrum-node --features madsim_compliant --test sim_consensus

# Disable blockchain indexing and transaction inclusion writes in vastrum-node/src/execution/execution.rs to get proper benchmarks
run_benchmark :; cd runtime/runtime-benchmark && cargo run --release

cli_install :; cargo install --path tooling/cli

get_lines_of_code_vastrum_node :; find vastrum-node/src -name '*.rs' | xargs wc -l

get_lines_of_code_total:
	find . -path '*/src/*.rs' -name '*.rs' \
	  -not -path '*/vendored-helios/*' \
	  -not -path '*/vendored-jmt-main/*' \
	  | xargs wc -l


SHELL := /bin/bash

# deploys blocker first, waits until it is fully deployed by checking domain registration, then deploys rest of the apps.
deploy-all-localnet:
	export VASTRUM_LOCALNET=1; \
	trap 'kill 0' SIGINT SIGTERM EXIT; \
	(cd apps/blocker && cargo run -p vastrum-cli -- run-dev) & \
	until curl -sf -X POST http://127.0.0.1:15556/resolvedomain/ \
		-H 'Content-Type: application/json' \
		-d '{"domain":"blocker"}' 2>/dev/null | grep -qv '"site_id":null'; do sleep 1; done; \
	(cd apps/chatter/deploy && cargo run) & \
	(cd apps/concord/deploy && cargo run) & \
	(cd apps/concourse/deploy && cargo run) & \
	(cd apps/gitter/deploy && cargo run) & \
	(cd apps/swapper/deploy && cargo run) & \
	(cd apps/letterer/deploy && cargo run) & \
	(cd apps/mapper/deploy && cargo run) & \
	(cd apps/vastrum-docs/deploy && cargo run) & \
	wait



deploy-all-production:
	(cd apps/blocker/deploy && cargo run); \
	until curl -sf -X POST https://rpc.vastrum.org/resolvedomain/ \
		-H 'Content-Type: application/json' \
		-d '{"domain":"blocker"}' 2>/dev/null | grep -qv '"site_id":null'; do sleep 1; done; \
	(cd apps/chatter/deploy && cargo run) & \
	(cd apps/concord/deploy && cargo run) & \
	(cd apps/concourse/deploy && cargo run) & \
	(cd apps/gitter/deploy && cargo run) & \
	(cd apps/swapper/deploy && cargo run) & \
	(cd apps/letterer/deploy && cargo run) & \
	(cd apps/mapper/deploy && cargo run) & \
	(cd apps/vastrum-docs/deploy && cargo run) & \
	wait