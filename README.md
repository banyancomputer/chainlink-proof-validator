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

# chainlink Node Setup

Follow the instructions to set up a chainlink EA Node on https://docs.chain.link/docs/running-a-chainlink-node/

Once you have set everything up, each time you start up your chainlink node again requires: 

Connect postgreSQl to workbench: sh sqlworkbench.sh
Pull desired docker image if not already present in docker: docker pull smartcontract/chainlink:1.7.0-root
Run: docker run -p 6688:6688 -v ~/.chainlink-goerli:/chainlink -it --env-file=.env smartcontract/chainlink:1.7.0-nonroot local n
Make sure your ip is permitted in inbound security group settings for AWS PostgreSql backend. Add your ip, which you can find with the command: dig -4 TXT +short o-o.myaddr.l.google.com @ns1.google.com 

To launch the API on the localhost: Cargo run 

Create a chainlink job by copying either the ea_job.toml of the example_job.toml into the chainlink node operator UI. Create a bridge in the UI, specifying the name of the bridge in the job (.i.e. rust_proof_verifier), and make sure to specify that the url is a docker internal address: http://host.docker.internal:8000/compute

# usage
Deploy contract using 
npx hardhat run scripts/deploy.js --network goerli

Trigger chainlink API contract function for basic testing using 
npx hardhat run scripts/test_ea.js --network goerli 

# testing

To test your Chainlink EA without constantly making calls to chain, use the unit testing functions in main. Note to use a single thread, since concurrency may give you problems with the nonce in your Eth Client. A better longterm solution would be to develop a nonce manager (Pull requests welcome!)

cargo test -- --test-threads=1

# Things to know 

Our implementation of the EA for our specific use case looks almost identical to the example EA, which can be found https://github.com/banyancomputer/chainlink-external-adapter-rs/tree/testing-setup. There, you have the option of using forge to deploy your contracts if you are more comfortable with that framework. 

Our implementation uses an EthClient as our provider for maing calls to chain. We imnplemented this as a wrapper that makes interfacing with the contract ABI making calls that require gas and modify state much easier. You can check out that implementation in https://github.com/banyancomputer/banyan-shared-rs/blob/master/src/eth.rs, THe ABI for the contract is inputted there, which can be found in hardhat_test/artifacts/contracts/Proofs.sol/Proofs.json after deploying the contract. 

