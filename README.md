# Secret Alias

Contract for anyone to create an alias for their secret network wallet address.
e.g. 'batman': "secret12345678901234567890"

## Concept / Why

Requesting payment requires opening your wallet, copying your address and sending it to the sender. By letting someone get your wallet address via an alias, it provides a fun, convenient alternative that reduces the chance for human error.

### Use case examples:

1. Standalone website 
- Batman asks Bane to send him 3 scrt tokens.
- Batman tells Bane that he can grab his wallet address from somewebsite.com by searching for 'thedarkknight'.
- Bane goes to website, searches for 'thedarkknight' and retrieves the wallet address.

2. Via dapp
- Bane asks the league of shadows to send him 333 tokens.
- The league of shadows is using somewallet which has incorporated this contract.
- Bane tells Ra's al Ghul that his alias is 'breakyou'.
- The league of shadows enters the alias into somewallet and it auto fills the wallet adddress to send to.

## Code examples

### Run locally

```sh
// 1. Setup local docker container to have local blockchain running.
// 2. Access container via separate terminal window
docker exec -it secretdev /bin/bash
// 3. cd into code folder
cd code
// 4. Store the contract (Specify your keyring. Mine is named test etc.)
secretcli tx compute store contract.wasm.gz --from a --gas 1000000 -y --keyring-backend test
// 5. Get the contract's id
secretcli query compute list-code
// 6. Initialize an instance of the contract
INIT='{"max_alias_size": 99}'
CODE_ID=1
secretcli tx compute instantiate $CODE_ID "$INIT" --from a --label "secret alias" -y --keyring-backend test
// 7. Check instance creation
secretcli query compute list-contract-by-code $CODE_ID
// 8. Grab the contract instance address from the last call
CONTRACT_INSTANCE_ADDRESS=secret********************
// 9. Create a new alias
secretcli tx compute execute $CONTRACT_INSTANCE_ADDRESS '{"create": { "alias_string": "emily" }}' --from a --keyring-backend test
// 10. Query the alias
secretcli query compute query $CONTRACT_INSTANCE_ADDRESS '{"show": { "alias_string": "emily"}}'
```

### Run on testnet

Same as above except you need to create a wallet and add tokens to it from the faucet. Specify the alias for that wallet when doing tx compute. In the examples below, I use my wallet on the testnet that I have aliased as 'testyyyy' locally.
```sh
// 1. Test
RUST_BACKTRACE=1 cargo unit-test
cargo integration-test
// 2. Generate schema
cargo schema
// 3. Compile wasm
cargo wasm
// 4. Optimize compiled wasm
docker run --rm -v $(pwd):/contract --mount type=volume,source=$(basename $(pwd))_cache,target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry enigmampc/secret-contract-optimizer
// 5. Start the docker environment with the contract:
docker run -it --rm -p 26657:26657 -p 26656:26656 -p 1337:1337 -v $(pwd):/root/code --name secretdev enigmampc/secret-network-sw-dev
// 6. Go into the docker environment in another terminal window
docker exec -it secretdev /bin/bash
// 7. Go into the code folder
cd code
// 8. Store the contract template into the blockchain
secretcli tx compute store contract.wasm.gz --from testyyyy -y --gas 1000000 --gas-prices=1.0uscrt
// 9. Create an instance of the contract
secretcli tx compute instantiate $CODE_ID "$INIT" --from testyyyy --label "secret alias" -y
// 10. Example of interacting with the contract
secretcli tx compute execute $CONTRACT_INSTANCE_ADDRESS '{"create": { "alias_string": "emily" }}' --from testyyyy
```
