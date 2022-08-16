require("@nomicfoundation/hardhat-toolbox");
require('dotenv').config()

/** @type import('hardhat/config').HardhatUserConfig */

const PRIVATE_KEY = process.env.PRIVATE_KEY

module.exports = {
  solidity: "0.8.9",
  defaultNetwork: "rinkeby",
  networks: {
     hardhat: {},
     rinkeby: {
        url: "https://rinkeby.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe",
        accounts: [PRIVATE_KEY]
     },
     goerli: {
        url: "https://goerli.infura.io/v3/1a39a4b49b9f4b8ba1338cd2064fe8fe",
        accounts: [PRIVATE_KEY]
     },
  },
}
