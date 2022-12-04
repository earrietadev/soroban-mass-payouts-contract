default:build

build:
	cargo build --target wasm32-unknown-unknown --release

build_logs:
	cargo build --target wasm32-unknown-unknown --profile release-with-logs

initialize_contract:
	soroban invoke \
        --wasm target/wasm32-unknown-unknown/release/mass_payouts.wasm \
        --id 1 \
        --fn initialize

#start_node:
#	docker run --rm -it \
#      --platform linux/amd64 \
#      -p 8000:8000 \
#      --name futurenet_stellar \
#      stellar/quickstart:soroban-dev@sha256:0993d3350148af6ffeab5dc8f0b835236b28dade6dcae77ff8a09317162f768d \
#      --futurenet \
#      --enable-soroban-rpc
#
#deploy_contract:
#	soroban deploy \
#        --wasm target/wasm32-unknown-unknown/release/mass_payouts.wasm \
#        --secret-key $(secret) \
#        --rpc-url http://localhost:8000/soroban/rpc \
#        --network-passphrase 'Test SDF Future Network ; October 2022'
#
#get_state:
#	soroban invoke \
#        --id be29f6e64897e3c5c1a5360b1d9dcfcc07a74c9b2b784cbf675c00f22e112fc6 \
#        --secret-key $(secret) \
#        --rpc-url http://localhost:8000/soroban/rpc \
#        --network-passphrase 'Test SDF Future Network ; October 2022' \
#        --fn initialize

test:
	cargo test -- --nocapture