default:build

build:
	cargo build --target wasm32-unknown-unknown --release

build_logs:
	cargo build --target wasm32-unknown-unknown --profile release-with-logs

initialize:
	soroban invoke \
        --wasm target/wasm32-unknown-unknown/release/mass_payouts.wasm \
        --id 1 \
        --fn initialize \
        --arg '{"object":{"vec":[{"symbol":"Account"},{"object":{"accountId":{"publicKeyTypeEd25519":"$(admin)"}}}]}}' \
        --arg $(currency)

get_state:
	soroban invoke \
		--wasm target/wasm32-unknown-unknown/release/mass_payouts.wasm \
		--id 1 \
		--fn get_state

set_acc:
	soroban invoke \
		--wasm target/wasm32-unknown-unknown/release/mass_payouts.wasm \
		--id 1 \
		--account $(admin) \
		--fn set_acc \
		--arg '{"object":{"accountId":{"publicKeyTypeEd25519":"$(account)"}}}' \
		--arg '{ "u32": $(amount) }' \

get_acc:
	soroban invoke \
		--wasm target/wasm32-unknown-unknown/release/mass_payouts.wasm \
		--id 1 \
		--fn get_acc \
		--arg '{"object":{"accountId":{"publicKeyTypeEd25519":"$(account)"}}}'

start_node:
	docker run --rm -it \
      --platform linux/amd64 \
      -p 8000:8000 \
      --name futurenet_stellar \
      stellar/quickstart:soroban-dev@sha256:0993d3350148af6ffeab5dc8f0b835236b28dade6dcae77ff8a09317162f768d \
      --futurenet \
      --enable-soroban-rpc

deploy_contract:
	soroban deploy \
        --wasm target/wasm32-unknown-unknown/release/mass_payouts.wasm \
        --secret-key $(secret) \
        --rpc-url http://localhost:8000/soroban/rpc \
        --network-passphrase 'Test SDF Future Network ; October 2022'

test:
	cargo test -- --nocapture