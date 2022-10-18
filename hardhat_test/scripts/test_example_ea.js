const hre = require("hardhat");
const {Buffer} = require('node:buffer');
const fs = require('fs');
const {TextEncoder} = require("util");
const { exit } = require("node:process");

async function main() {
  
  const Test = await ethers.getContractFactory("BlockTime");
  const test = await Test.attach("0x8BfB349916287410C2e195768f389a81100126ef");
  console.log(test.address);

  let jobId = 'a66c9947b4d94331ae8fd445265bf430';
  const transactionResponse_2 = await test.startComputeTimeSinceWithChainlink(0,jobId);
  const transactionReceipt_2 = await transactionResponse_2.wait(); 
  console.log(transactionResponse_2)
  

  const transactionResponse3 = await test.timeSince();
  console.log("Time Since:", transactionResponse3);
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });



