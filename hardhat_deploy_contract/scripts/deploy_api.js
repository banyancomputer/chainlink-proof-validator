const hre = require("hardhat");
const {Buffer} = require('node:buffer');
const fs = require('fs');
const {TextEncoder} = require("util");
const { exit } = require("node:process");

async function main() {

    await hre.run("compile")
    const Api = await hre.ethers.getContractFactory("APIConsumer");
    const api = await Api.deploy();
    await api.deployed();
    console.log(api.address);
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
