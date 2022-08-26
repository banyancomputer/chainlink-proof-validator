const hre = require("hardhat");
const {Buffer} = require('node:buffer');
const fs = require('fs');
const {TextEncoder} = require("util");
const { exit } = require("node:process");

async function main() {
  
  const Proofs = await ethers.getContractFactory("Proofs");
  const proofs = await Proofs.attach("0x9ee596734485268eF62db4f3E61d891E221504f6");
  console.log(proofs.address);

  let offerId = 55378008;
  let deal_start_block = 0; 
  let deal_length_in_blocks = 10; 
  let proof_frequency_in_blocks = 5; 
  let price = 0; 
  let collateral = 0; 
  let erc20_token_denomination = '0x326C977E6efc84E512bB9C30f76E30c160eD06FB'; // addr 
  var ipfs_file_cid = "Qmd63gzHfXCsJepsdTLd4cqigFa7SuCAeH6smsVoHovdbE"; 
  var file_size = 920000; 
  var blake3_checksum = "c1ae1d61257675c1e1740c2061dabfeded7575eb27aea8aa4eca88b7d69bd64f"; 

  const transactionResponse_2 = await proofs.createOffer({"offerId": offerId, "deal_start_block": deal_start_block, "deal_length_in_blocks": deal_length_in_blocks, "proof_frequency_in_blocks": proof_frequency_in_blocks, "price": price, "collateral": collateral, "erc20_token_denomination": erc20_token_denomination, "ipfs_file_cid": ipfs_file_cid, "file_size": file_size, "blake3_checksum": blake3_checksum});
  const transactionReceipt_2 = await transactionResponse_2.wait(); 
  console.log("done")
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});