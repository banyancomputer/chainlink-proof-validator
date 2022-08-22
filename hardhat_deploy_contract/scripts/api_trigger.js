const hre = require("hardhat");
const ethers = require("ethers")
const {Buffer} = require('node:buffer');
const fs = require('fs');
const {TextEncoder} = require("util");
const { exit } = require("node:process");

async function main() {
  
  await hre.run("compile")
  let address = "0x042e74404ccE32d6198f9b228E58b06b9659A44F"
  let provider = new ethers.providers.JsonRpcProvider("https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe");
  console.log(await provider.getCode(address)) 
  // Code does exist. 

  const Api = await hre.ethers.getContractFactory("APIConsumer");
  const api = await Api.attach(address);

  const transactionResponse = await api.requestVolumeData();
  const transactionReceipt = await transactionResponse.wait()
  console.log(transactionReceipt)
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
