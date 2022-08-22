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
  const bao = await Bao.attach("0x464cBd3d0D8A2872cf04306c133118Beb5711111"); // WRONG 

  const transactionResponse = await bao.save_proof(proof);
  const transactionReceipt = await transactionResponse.wait()
  console.log(transactionReceipt)
  console.log(transactionReceipt.events[0].args)
}