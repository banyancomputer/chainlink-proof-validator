const { expect } = require("chai");
const hre = require("hardhat");
const fs = require('fs');
var mocha = require('mocha')
const axios = require('axios').default;
var assert = require('assert');


describe("Integration basic", async function () {

  var contract;
  var proofs = [];
  var deal_start_block;
  var deal_length_in_blocks = 10; 
  var proof_frequency_in_blocks = 5; 
  var price = 0; 
  var collateral = 0; 
  var erc20_token_denomination = '0x326C977E6efc84E512bB9C30f76E30c160eD06FB'; // addr 
  var ipfs_file_cid = "Qmd63gzHfXCsJepsdTLd4cqigFa7SuCAeH6smsVoHovdbE"; 
  var file_size = 941366; 
  var blake3_checksum = "c1ae1d61257675c1e1740c2061dabfeded7575eb27aea8aa4eca88b7d69bd64f"; 
  var offerId = 2;

  const latestBlock = await hre.ethers.provider.getBlock("latest")
  console.log("current_block:" , latestBlock.number);

  let diff = latestBlock.number % proof_frequency_in_blocks;
  if (diff == 0)
  {
    diff = proof_frequency_in_blocks;
  }
  let target_window = latestBlock.number - diff;
  console.log("target_block:" ,target_window);

  var deal_start_block = target_window - proof_frequency_in_blocks;
  console.log("deal_start_block:" , deal_start_block);

  // deal_info = lookup(deal_id).handle_error()
  // if msg.sender != deal_info.service_provider_addr
  // { revert () }
  assert(target_window >= deal_start_block, "target_window must be greater than deal_start_block");
  let offset = target_window - deal_start_block;
  assert(offset < deal_length_in_blocks);
  assert(offset % proof_frequency_in_blocks == 0); 
  let window_num = offset / proof_frequency_in_blocks;

  assert (latestBlock.number > target_window, "latestBlock.number must be greater than target_window");
  assert (latestBlock.number <= target_window + proof_frequency_in_blocks, "latestBlock.number must be less than target_window + proof_frequency_in_blocks");

  // if (deal_info.proofs[window_num] != null)
  // { revert () }

  // proof_blocks[offer_id][window_num] = block_number
  // emit(deal_id, window_num, proof)

  
  it("simulates correct target_window", async function() {
    // runs before all tests in this file regardless where this line is defined.
    //const Contract = await hre.ethers.getContractFactory("Proofs");
    //contract = await Contract.attach("0xeb3d5882faC966079dcdB909dE9769160a0a00Ac");
    //console.log(contract.address);



  });

/*
    let txtFileGood = "bao_slice_good.txt";
    let proof_good = fs.readFileSync(txtFileGood);
    proofs.push(proof_good);

    let txtFileBad = "bao_slice_bad.txt";
    let proof_bad = fs.readFileSync(txtFileBad);
    proofs.push(proof_bad);
  });

  before(async function() {
    // runs before each test in this block
    this.timeout(1000000);
    let target_window = 0;
    const transactionResponse_2 = await contract.createOffer({"offerId": offerId, "deal_start_block": deal_start_block, "deal_length_in_blocks": deal_length_in_blocks, "proof_frequency_in_blocks": proof_frequency_in_blocks, "price": price, "collateral": collateral, "erc20_token_denomination": erc20_token_denomination, "ipfs_file_cid": ipfs_file_cid, "file_size": file_size, "blake3_checksum": blake3_checksum});
    const transactionReceipt_2 = await transactionResponse_2.wait(); 

    let indices = [];
    if (offerId == 0) {
      indices = [0,0]
    }
    else if (offerId == 1) {
      indices = [1,1]
    }
    else if (offerId == 2) {
      indices = [0,1]
    }

    // proofs are added with offer id and window num
    for (let i = 0; i < proofs.length; i++) {
      const transactionResponse = await contract.save_proof(proofs[indices[i]], offerId, i);
      const transactionReceipt = await transactionResponse.wait()
      console.log("window_num_logged:", i)
    }
  });

  it("Access testnet deployed contract", async function () {
    
    //console.log(owner)
    const transactionResponse = await contract.getDeal(offerId);
    console.log(transactionResponse)
    expect(transactionResponse.offerId).to.equal(offerId);
  });

  it("Access multiple proofs", async function () {
    console.log("offerId: " + offerId)
    const proof_1 = await contract.getProofBlock(offerId, 0);
    console.log("proof_1:",  proof_1)

    const proof_2 = await contract.getProofBlock(offerId, 1);
    console.log("Proof_2:", proof_2)
  
    expect(proof_1).to.not.equal(proof_2);
  });

  it ("API call", async function () {
    console.log("offerId: " + offerId)
    await axios({
      method: "post",
      url: "http://127.0.0.1:8000/val",
      data: {
        "job_run_id": "12345", 
        "data": 
        {
            "offer_id": offerId.toString()
        }
      },
      headers: { "Content-Type": "application/json" },
    })
      .then(function (response) {
        //handle success
        console.log(response.data);
      })
      .catch(function (response) {
        //handle error
        console.log(response);
      });

  });

  afterEach(function() {
});


});