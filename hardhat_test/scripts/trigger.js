const hre = require("hardhat");
const {Buffer} = require('node:buffer');
const fs = require('fs');
const {TextEncoder} = require("util");
const { exit } = require("node:process");

async function main() {
  
  await hre.run("compile")
  const Lock = await ethers.getContractFactory("Lock");
  const lock = await Lock.attach("0x464cBd3d0D8A2872cf04306c133118Beb5711111"); // WRONG 

  let offerId = 55378008;
  let deal_start_block = 2; 
  let deal_length_in_blocks = 3; 
  let proof_frequency_in_blocks = 4; 
  let price = 5; 
  let collateral = 6; 
  let erc20_token_denomination = '0xf679d8d8a90f66b4d8d9bf4f2697d53279f42bea'; // addr 
  let ipfs_file_cid = 8; 
  let file_size = 9; 
  let blake3_checksum = 10; 
  let proof_blocks = [];

  const transactionResponse_2 = await lock.createOffer(offerId, deal_start_block, deal_length_in_blocks, proof_frequency_in_blocks, price, collateral, erc20_token_denomination, ipfs_file_cid, file_size, blake3_checksum, proof_blocks);
  const transactionReceipt_2 = await transactionResponse_2.wait(); 
  console.log(transactionReceipt_2)
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});