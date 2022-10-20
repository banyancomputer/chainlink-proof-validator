const hre = require("hardhat");
const fs = require('fs');

async function main() {

  const Test = await ethers.getContractFactory("Proofs");
  const test = await Test.attach("0x8185599b47373dF84CB504cbfA124295FF4F346e");
  console.log(test.address);

  let jobId = '8a1513779ab24089b7d6a52fb36dfb41';
  const transactionResponse_2 = await test.requestVerification(jobId,"100","1");
  const transactionReceipt_2 = await transactionResponse_2.wait(); 
  console.log(transactionResponse_2)
}
main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });