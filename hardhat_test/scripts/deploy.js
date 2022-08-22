// We require the Hardhat Runtime Environment explicitly here. This is optional
// but useful for running the script in a standalone fashion through `node <script>`.
//
// You can also run a script with `npx hardhat run <script>`. If you do that, Hardhat
// will compile your contracts, add the Hardhat Runtime Environment's members to the
// global scope, and execute the script.
const hre = require("hardhat");
const fs = require('fs');

async function main() {

  const Lock = await hre.ethers.getContractFactory("Lock");
  const lock = await Lock.deploy();

  await lock.deployed();

  let txtFile = "bao_slice_2.txt";
  let proof = fs.readFileSync(txtFile);
  const transactionResponse = await lock.save_proof(proof);
  const transactionReceipt = await transactionResponse.wait()
  console.log(transactionReceipt)
  console.log(transactionReceipt.events[0].args)
  
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

  const transactionResponse_2 = await lock.createOffer(offerId, deal_start_block, deal_length_in_blocks, proof_frequency_in_blocks, price, collateral, erc20_token_denomination, ipfs_file_cid, file_size, blake3_checksum);
  const transactionReceipt_2 = await transactionResponse_2.wait(); 
  console.log(transactionReceipt_2)
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
