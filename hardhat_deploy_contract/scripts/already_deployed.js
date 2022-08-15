const hre = require("hardhat");
const {Buffer} = require('node:buffer');
const fs = require('fs');
const {TextEncoder} = require("util");
const { exit } = require("node:process");

async function main() {

  let txtFile = "bao_slice_2.txt";
  let proof = fs.readFileSync(txtFile);
  
  await hre.run("compile")
  const Bao = await ethers.getContractFactory("Bao");
  const bao = await Bao.attach("0xf679d8d8a90f66b4d8d9bf4f2697d53279f42bea");

  const transactionResponse = await bao.save_proof(proof);
  const transactionReceipt = await transactionResponse.wait()
  console.log(transactionReceipt)
  console.log(transactionReceipt.events[0].args)
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
