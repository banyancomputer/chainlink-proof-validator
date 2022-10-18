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
Make sure ip is permitted in inbound security group settings for AWS PostgreSql backend. Find ip: dig -4 TXT +short o-o.myaddr.l.google.com @ns1.google.com and then add. 

#testing

cargo test --bin deploy -- --test-threads=1
