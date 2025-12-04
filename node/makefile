#single node environemnts
network_testnet :; cargo build && make -j 5 node5 node4 node3 node2 node1 
network_prod :; cargo build --release && make -j 5 node5_prod node4_prod node3_prod node2_prod node1_prod 

network_testnet_freeze_nodes_randomly :; FREEZE_NODES_RANDOMLY=TRUE make network_prod


node1 :; sleep 0 && NODE_ID=1 cargo run
node2 :; sleep 1 && NODE_ID=2 cargo run
node3 :; sleep 2 && NODE_ID=3 cargo run
node4 :; sleep 3 && NODE_ID=4 cargo run
node5 :; sleep 4 && NODE_ID=5 cargo run

node1_prod :; sleep 0 && NODE_ID=1 ../target/release/node
node2_prod :; sleep 1 && NODE_ID=2 ../target/release/node
node3_prod :; sleep 2 && NODE_ID=3 ../target/release/node
node4_prod :; sleep 3 && NODE_ID=4 ../target/release/node
node5_prod :; sleep 4 && NODE_ID=5 ../target/release/node

get_lines_of_code :; find src -name \*.rs | xargs wc -l
