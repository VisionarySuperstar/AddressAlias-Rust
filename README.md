# Address Alias - WORK IN PROGRESS

Create an alias for your address. Search an address by alias. Search an alias by address.
"batman" => "secret12345678901234567890"
"secret12345678901234567890" => "batman"

## Concept / Why

- Easy way to find and share addresses.
- Easy way to let people know the purpose of an address. e.g. "btn.group - admin 1"

### Use case examples:

1. Finding and sharing an address through btn.group's site
- Batman asks Bane to send him 3 scrt tokens.
- Batman tells Bane that he can grab his wallet address from btn.group[https://www.btn.group/secret_network/address_alias] by searching for 'thedarkknight'.
- Bane goes to website, searches for 'thedarkknight' and retrieves the wallet address.

2. Finding and sharing an address via a third party app
- Bane asks the league of shadows to send him 333 tokens.
- The league of shadows is using an app which has incorporated this contract.
- Bane tells the league of shadows that his alias is 'breakyou'.
- The league of shadows enters the alias into the app and it auto fills the wallet adddress to send to.

3. Easier auditing
- Robin wants to audit an instance of a smart contract before investing.
- Robin sees a few different addresses interacting with it.
- He looks up an address in a flash and gets a better picture of what's going on.

## Code examples

### Before deploying to blockchain

```sh
// 1. Run tests
RUST_BACKTRACE=1 cargo unit-test
cargo integration-test

// 2. Generate schema
cargo schema

// 3. Compile wasm
cargo wasm

// 4. Optimize compiled wasm
docker run --rm -v $(pwd):/contract --mount type=volume,source=$(basename $(pwd))_cache,target=/code/target --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry enigmampc/secret-contract-optimizer
```

### When running on local development blockchain

- Make sure you do the things in 'Before deploying to blockchain' first.

```sh
// 1. Setup local docker container to run devleopment blockchain
docker run -it --rm -p 26657:26657 -p 26656:26656 -p 1337:1337 -v $(pwd):/root/code --name secretdev enigmampc/secret-network-sw-dev

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

### When running on community testnet

- Make sure you do the things in 'Before deploying to blockchain' first.
- Make sure you create a wallet on the community testnet first and add some tokens from the faucet. Check out 'Secret contracts guide' link below.
- You may also need to specify the testnet url etc as well. Check that 'Secret contracts guide'.
- Specify the alias for that wallet when doing tx compute. In the examples below, I use my wallet on the testnet that I have aliased as 'testyyyy' locally.

```sh
// 1. Store the contract template into community testnet
secretcli tx compute store contract.wasm.gz --from testyyyy -y --gas 1000000 --gas-prices=1.0uscrt

// 2. Get the contract's id
secretcli query compute list-code

// 3. Store desired contract id into variable
CODE_ID=**INSERT DESIRED CODE ID RETRIEVED FROM STEP 2**

// 4. Create an instance of the contract
INIT='{"max_alias_size": 99}'
secretcli tx compute instantiate $CODE_ID "$INIT" --from testyyyy --label "secret alias" -y

// 5. Check instance creation
secretcli query compute list-contract-by-code $CODE_ID

// 6. Store desired contract instance address into a variable
CONTRACT_INSTANCE_ADDRESS=secret1zdh7d9gg82pt6uh3yp3ewdu990mjfva8ceyes9

// 7. Example of interacting with the contract
secretcli tx compute execute $CONTRACT_INSTANCE_ADDRESS '{"create": { "alias_string": "emily" }}' --from testyyyy
```

### When running on production testnet

- I was provided two options to do this, run my own node or use Datahub Figment.
- I can't run my own node because I can't get SGX to work on my macbook pro.
- Will use Figment for the meanwhile and re-evaluate later.

1. Sign up to Figment datahub
2. Grab url with your api key in it from site
3. Create a .env in the root file
4. Add .env to .gitignore
5. Add details to .env
```sh
SECRET_REST_URL=https://secret-2--lcd--full.datahub.figment.io/apikey/<your key here>/
SECRET_CHAIN_ID=secret-2
ADDRESS=<your secretaddress that has some scrt tokens in it>
MNEMONIC=<your secretaddress' mnemonic>
```
6. Initialize as an npm folder in terminal
```sh
npm init -y
```
7. Install required packages
```sh
npm install --save secretjs dotenv @iov/crypto
```
8. Unzip optimized contract as SecretJS expect a .wasm file (secretcli accepts the optimized version)
```sh
gunzip contract.wasm.gz
```
9. Put js files into root folder and run in terminal e.g.
```sh
node deploy.js
```

## References
1. Secret contracts guide: https://github.com/enigmampc/secret-contracts-guide
