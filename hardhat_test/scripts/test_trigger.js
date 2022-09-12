const hre = require("hardhat");
const {Buffer} = require('node:buffer');
const fs = require('fs');
const {TextEncoder} = require("util");
const { exit } = require("node:process");

async function main() {
  
  const Test = await ethers.getContractFactory("Test");
  const test = await Test.attach("0x392f174AD59860946C4480eCf51fBF8b22eD05cE");
  console.log(test.address);

  const transactionResponse_2 = await test.getDeal({_offerId: 0})
  const transactionReceipt_2 = await transactionResponse_2.wait(); 
    console.log(transactionResponse_2)
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });



