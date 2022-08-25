const hre = require("hardhat");
const fs = require('fs');

describe("Proof Verifier", function() {
  var contract;
  var proofs = [];
  var deal_start_block = 0; 
  var deal_length_in_blocks = 10; 
  var proof_frequency_in_blocks = 5; 
  var price = 0; 
  var collateral = 0; 
  var erc20_token_denomination = '0x326C977E6efc84E512bB9C30f76E30c160eD06FB'; // addr 
  var ipfs_file_cid = "Qmd63gzHfXCsJepsdTLd4cqigFa7SuCAeH6smsVoHovdbE"; 
  var file_size = 920000; 
  var blake3_checksum = "c1ae1d61257675c1e1740c2061dabfeded7575eb27aea8aa4eca88b7d69bd64f"; 
  var offerId = 0;

  beforeAll(async function() {
    const Contract = await hre.ethers.getContractFactory("Proofs");
    contract = await Contract.attach("0x9ee596734485268eF62db4f3E61d891E221504f6");
    console.log(contract.address);

    let txtFileGood = "bao_slice_good.txt";
    let proof_good = fs.readFileSync(txtFileGood);
    proofs.push(proof_good);

    let txtFileBad = "bao_slice_bad.txt";
    let proof_bad = fs.readFileSync(txtFileBad);
    proofs.push(proof_bad);

  });
  // This will override the entries in eth memory each time I test. 
  beforeEach(async function() {

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

    for (let i = 0; i < proofs.length + 1; i++) {
      const transactionResponse = await contract.save_proof(proofs[indices[i]], offerId, target_window);
      const transactionReceipt = await transactionResponse.wait()
      target_window+=5;
    }

  }, 100000);

  it("contains spec with an expectation", function() {
    console.log("zero", offerId)
    var xhr = new XMLHttpRequest();
    xhr.open("POST", yourUrl, true);
    xhr.setRequestHeader('Content-Type', 'application/json');
    xhr.send(JSON.stringify({
      job_run_id: "49f9283dfce34ca1b60b59d0291e82a8", 
      data: {
          "offer_id": offerId,
      }
    }));
    console.log("response", xhr.responseText);
    expect(true).toBe(true);
  }, 100000);

  it("contains spec with an expectation", function() {
    console.log("one" , offerId)
    expect(false).toBe(false);
  }, 100000);

  afterEach(function() {
    offerId++;
  });
});
