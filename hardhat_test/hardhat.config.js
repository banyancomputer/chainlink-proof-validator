require("@nomicfoundation/hardhat-toolbox");
require('dotenv').config()

/** @type import('hardhat/config').HardhatUserConfig */

const PRIVATE_KEY = process.env.PRIVATE_KEY

module.exports = {
  solidity: {
      compilers: [
        {
          version: "0.8.13",
        },
        {
          version: "0.7.0",
          settings: {},
        },
      ],
   },
     settings: {
         optimizer: {
         enabled: true,
         runs: 200,
         details: {
            yul: false
         }
   },
  },
  defaultNetwork: "rinkeby",
  networks: {
     hardhat: {},
     rinkeby: {
        url: "https://rinkeby.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe",
        accounts: [PRIVATE_KEY]
     },
     goerli: {
        url: "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe",
        accounts: [PRIVATE_KEY],
        gas: 2000000
     },
  },
}