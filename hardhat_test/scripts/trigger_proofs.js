const hre = require("hardhat");
const fs = require('fs');

async function main() {

  const Proofs = await ethers.getContractFactory("Proofs");
  const proofs = await Proofs.attach("0x3bAbD1bb8B6eAb26Bd6d837d92659F6dE6a63d58");
  console.log(proofs.address);

  const transactionResponse3 = await proofs.verification();
  console.log("Verification Before:", transactionResponse3);

  const transactionResponse = await proofs.requestVerification("5b0afcd653034b7b9a39457087e2ac2c","7457561", "55378008");
  const transactionReceipt = await transactionResponse.wait()
  console.log(transactionReceipt);

  // wait 30 seconds
  await new Promise(r => setTimeout(r, 100000));
  // need to be checking for an event here since the reciept comes way before the chainlink settles up 
  const transactionResponse2 = await proofs.verification();
  console.log("Verification After:", transactionResponse2);

  /*
  let offerId = 55378008;
  let deal_start_block = 2; 
  let deal_length_in_blocks = 3;
  let proof_frequency_in_blocks = 4; 
  let price = 5; 
  let collateral = 6; 
  let erc20_token_denomination = '0xf679d8d8a90f66b4d8d9bf4f2697d53279f42bea'; // addr 
  let ipfs_file_cid = "Qmd63gzHfXCsJepsdTLd4cqigFa7SuCAeH6smsVoHovdbE"; 
  let file_size = 9; 
  let blake3_checksum = "c1ae1d61257675c1e1740c2061dabfeded7575eb27aea8aa4eca88b7d69bd64f"; 
  let proof_blocks = []

  const transactionResponse_2 = await proofs.createOffer({"offerId": offerId, "deal_start_block": deal_start_block, "deal_length_in_blocks": deal_length_in_blocks, "proof_frequency_in_blocks": proof_frequency_in_blocks, "price": price, "collateral": collateral, "erc20_token_denomination": erc20_token_denomination, "ipfs_file_cid": ipfs_file_cid, "file_size": file_size, "blake3_checksum": blake3_checksum, "proof_blocks": proof_blocks});
  const transactionReceipt_2 = await transactionResponse_2.wait(); 
  console.log(transactionReceipt_2)

  let txtFile = "bao_slice_2.txt";
  let proof = fs.readFileSync(txtFile);
  const transactionResponse = await proofs.save_proof(proof, 55378008);
  const transactionReceipt = await transactionResponse.wait()
  console.log(transactionReceipt)
  console.log(transactionReceipt.events[0].args)
  */
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});