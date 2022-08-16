// SPDX-License-Identifier: MIT
pragma solidity ^0.8.7;

import '@chainlink/contracts/src/v0.8/ChainlinkClient.sol';
import '@chainlink/contracts/src/v0.8/ConfirmedOwner.sol';


contract APIConsumer is ChainlinkClient, ConfirmedOwner {
    using Chainlink for Chainlink.Request;

    string public proof;
    bytes32 private jobId;
    uint256 private fee;

    event RequestProof(bytes32 indexed requestId, string proof);

    /**
     * @notice Initialize the link token and target oracle
     *
     * Goerli Testnet details:
     * Link Token: 0x326C977E6efc84E512bB9C30f76E30c160eD06FB
     * Oracle: 0xCC79157eb46F5624204f47AB42b3906cAA40eaB7 (Chainlink DevRel)
     * jobId: ca98366cc7314957b8c012c72f05aeeb
     *
     */
    constructor() ConfirmedOwner(msg.sender) {
        setChainlinkToken(0x326C977E6efc84E512bB9C30f76E30c160eD06FB); // Goerli Testnet LINK Token
        setChainlinkOracle(0xCC79157eb46F5624204f47AB42b3906cAA40eaB7); // Goerli oracle 
        jobId = '7d80a6386ef543a3abb52817f6707e3b'; // GET>string
        fee = (1 * LINK_DIVISIBILITY) / 10; // 0,1 * 10**18 (Varies by network and job)
    }

    /**
     * Create a Chainlink request to retrieve API response, find the target
     * data, then multiply by 1000000000000000000 (to remove decimal places from data).
     */
    function requestVolumeData() public returns (bytes32 requestId) {
        Chainlink.Request memory req = buildChainlinkRequest(jobId, address(this), this.fulfill.selector);

        // get adaptor to get binary 
        req.add('get', 'http://127.0.0.1:8000/validate/11189427');

        // task adaptor to parse json 
        req.add('path', 'data,Valid,result'); // Chainlink nodes 1.0.0 and later support this format

        // Sends the request
        return sendChainlinkRequest(req, fee);
    }

    /**
     * Receive the response in the form of uint256
     */
    function fulfill(bytes32 _requestId, string memory _proof) public recordChainlinkFulfillment(_requestId) {
        emit RequestProof(_requestId, _proof);
        proof = _proof;
    }

    /**
     * Allow withdraw of Link tokens from the contract
     */
    function withdrawLink() public onlyOwner {
        LinkTokenInterface link = LinkTokenInterface(chainlinkTokenAddress());
        require(link.transfer(msg.sender, link.balanceOf(address(this))), 'Unable to transfer');
    }
}