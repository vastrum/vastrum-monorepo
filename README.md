### Vastrum is a P2P website hosting protocol.

### See [documentation](https://vastrum.net/vastrum-docs/introduction) for more info (hosted on vastrum)

# Installation

### Need to install rust

    https://rust-lang.org/tools/install/

### Need to install npm

    https://github.com/nvm-sh/nvm


### Need libclang for libsrocksdb-sys

    sudo apt-get install clang

### Need wasm-pack

    https://drager.github.io/wasm-pack/installer/


### To run node

    make run_network_dev

### To run front end

    make run_frontend


### To stresstest consensus with randomly frozen nodes

    make run_network_freeze_nodes_randomly
