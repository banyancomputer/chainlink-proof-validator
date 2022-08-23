const hre = require("hardhat");
const fs = require('fs');

async function main() {

  const Operator = await hre.ethers.getContractFactory("Operator");
  const operator = await Operator.deploy("0x326C977E6efc84E512bB9C30f76E30c160eD06FB","0x8A4E8e012a5B9EC7817a7936e41DcD84489CE5ed");
  
  await operator.deployed();
  console.log("Proofs deployed to:", operator.address);

  let tx = await operator.setAuthorizedSenders(["0xecaa2bFc59eF156cAC6e126EBDc1D81cf78fCA36"]);

}
main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
  });