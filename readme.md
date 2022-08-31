# setup
install deps
```bash
$ npm i
```

move .env.example to .env
```bash
$ mv .env.example .env
```
then add your private key and infura API key to .env


# usage
see hardhat_deploy_contract for logging proof 

# chainlink Node Setup
Connect postgreSQl to workbench: sh sqlworkbench.sh
Pull desired docker image: docker pull smartcontract/chainlink:1.7.0-root
Run: docker run -p 6688:6688 -v ~/.chainlink-goerli:/chainlink -it --env-file=.env smartcontract/chainlink:1.7.0-nonroot local n

