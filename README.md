# What is it?

This is a Chainlink External Adaptor written in Rust that computes merkle proofs over deal histories to check if files are accurately being stored over IPFS. 

# What it does? 

It receives calls from a contract on-chain, makes calls to an API, does proof checking, and returns a success rate to chain. 

# initial setup
install deps
```bash
$ npm i
```

move .env.example to .env
```bash
$ mv .env.example .env
```
then add your private key and infura API key to .env

# PreReqs

Follow the instructions to set up a Chainlink External Adaptor Node on your local machine in a Docker on https://docs.chain.link/docs/running-a-chainlink-node/

You will need to install Docker, Sqlworkbench, and set up and AWS postgreSQL database. The instructions linked above walk you through this. You will also need to install Hardhat : 
```bash
npm i hardhat
```

# Chainlink Node Setup

Once you have set everything up, each time you start up your Chainlink node again requires: 

Connect postgreSQl to workbench:
```bash
$ sh sqlworkbench.sh
```
Pull desired docker image if not already present in docker: 
```bash
$ docker pull smartcontract/chainlink:1.7.0-root
```
Run: 
```bash
$ docker run -p 6688:6688 -v ~/.chainlink-goerli:/chainlink -it --env-file=.env smartcontract/chainlink:1.7.0-nonroot local n
```
Make sure your ip is permitted in inbound security group settings for AWS PostgreSql backend. Add your ip, which you can find with the command: 
```bash
$ dig -4 TXT +short o-o.myaddr.l.google.com @ns1.google.com 
```
To launch the API on the localhost: 
```bash
$ Cargo run 
```
Create a Chainlink job by copying the example_job.toml into the Chainlink node operator UI. Create a bridge in the UI, specifying the name of the bridge in the job (.i.e. rust_proof_verifier), and make sure to specify that the url is a docker internal address: http://host.docker.internal:8000/compute

You must deploy the operator.sol contract using the deploy_operator function and call the set_authorized_senders function. You can do by subsituting your own node address as the authorized sender when you call this script below
```bash 
$ npx hardhat run scripts/deploy_operator.js --network goerli
```

# contract deployment
Deploy contract using
```bash 
$ npx hardhat run scripts/deploy.js --network goerli
```
BlockTime.sol is a test contract. You can see our real contracts at https://github.com/banyancomputer/contracts. BlockTime.sol is deployed at 0x8185599b47373dF84CB504cbfA124295FF4F346e. 

Trigger Chainlink API contract function for basic testing using 
```bash
$ npx hardhat run scripts/test_example_ea.js --network goerli 
```
Make sure your contract is funded with some testnet link which you can get here https://faucets.chain.link/
# testing

To test your Chainlink External Adaptor without constantly making calls to chain, use the unit testing functions in main. Uncomment them out, and make sure you have your infura API_KEY in your env file. Note to use a single thread, since concurrency may give you problems with the nonce in your Eth Client. A better longterm solution would be to develop a nonce manager (Pull requests welcome!)

```bash
cargo test -- --test-threads=1
```

# Things to know 

Our implementation of the External Adaptor for our specific use case looks almost identical to the example External Adaptor, which can be found https://github.com/banyancomputer/chainlink-external-adapter-rs/tree/testing-setup. 

Our implementation uses an EthClient as our provider for maing calls to chain. We imnplemented this as a wrapper that makes interfacing with the contract ABI making calls that require gas and modify state much easier. You can check out that implementation in https://github.com/banyancomputer/banyan-shared-rs/blob/master/src/eth.rs, The ABI for the contract is inputted there, which can be found in hardhat_test/artifacts/contracts/Proofs.sol/Proofs.json after deploying the contract. 
