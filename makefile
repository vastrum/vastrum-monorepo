DEV_RPC_URL = http://127.0.0.1:3000/
PROD_RPC_URL = https://vastrum.org/

#to run a 5 node network locally
#run_network_dev :; make build && cd node && RUST_BACKTRACE=1 && make network_testnet
run_network_dev :; make build && cd node && RUST_BACKTRACE=1 && make network_prod
run_network_freeze_nodes_randomly :; make build && cd node && RUST_BACKTRACE=1 && make network_testnet_freeze_nodes_randomly

run_tests :; cd node && cargo test -- --test-threads=1

get_lines_of_code :; cd node && make get_lines_of_code

build :; make build_application && make install_deps

#run frontend locally
run_frontend :; \
	make build_application-lib && \
	make build_wasm_dev && \
	make install_deps && \
	cd web-client && npm run dev

#to deploy
build_dist_frontend :; \
	make build_application-lib && \
	make build_wasm_prod && \
	make install_deps && \
	cd web-client && npm run build


build_application :; \
	make install_deps && \
	(cd apps/forum && make build) && \
	(cd apps/static-deployer && make build)

build_application-lib :; \
	(cd application-lib && npm i && npm run build)

build_wasm_prod :; cd web-client/wasm && FRONTEND_RPC_URL=$(PROD_RPC_URL)  wasm-pack build --target bundler
build_wasm_dev :; cd web-client/wasm && FRONTEND_RPC_URL=$(DEV_RPC_URL)  wasm-pack build --target bundler

install_deps :; \
	(cd application-lib && npm i) && \
	(cd compiler && npm i) && \
	(cd web-client && npm i)

#generates types for webassembly host communication
gentstypes :; jco guest-types -o generated/types/guest/import webassembly/component-bindings.wit --world-name hostbindings
