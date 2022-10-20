// SPDX-License-Identifier: MIT
pragma solidity ^0.8.7;

import '@chainlink/contracts/src/v0.8/ChainlinkClient.sol';
import '@chainlink/contracts/src/v0.8/ConfirmedOwner.sol';

contract Proofs is ChainlinkClient, ConfirmedOwner {
    using Chainlink for Chainlink.Request;
    uint256 private fee;

    constructor() ConfirmedOwner(msg.sender) {
        setChainlinkToken(0x326C977E6efc84E512bB9C30f76E30c160eD06FB);
        setChainlinkOracle(0xF1a252307Ff9F3fbB9598c9a181385122948b8Ae);
        fee = (1 * LINK_DIVISIBILITY) / 10; // 0,1 * 10**18 (Varies by network and job)
    }
    
    enum OfferStatus { NON, OFFER_CREATED, OFFER_ACCEPTED, OFFER_ACTIVE, OFFER_COMPLETED, OFFER_FINALIZED, OFFER_TIMEDOUT, OFFER_CANCELLED }


    struct OfferCounterpart {
        uint256 amount;
        address partyAddress;
        bool cancel;
    }
    uint256 private _offerId;
    struct Deal {
        uint256 dealStartBlock;
        uint256 dealLengthInBlocks;
        uint256 proofFrequencyInBlocks;
        uint256 price;
        uint256 collateral;
        address erc20TokenDenomination;
        string ipfsFileCID; 
        uint256 fileSize;
        string blake3Checksum;
        OfferCounterpart creatorCounterpart;
        OfferCounterpart providerCounterpart;
        OfferStatus offerStatus;
    }
    
    mapping(uint256 => Deal) public _deals;
    mapping(uint256 => mapping (uint256 => uint256)) public _proofblocks; // offerID => (proofWindowCount => block.number)
    mapping(address => uint256[]) internal _openOffers;

    struct ResponseData {
        uint256 offer_id;
        uint256 success_count;
        uint256 num_windows;
        uint256 status;
        string result;
    }
    mapping(uint256 => ResponseData) public responses;

    event RequestVerification(bytes32 indexed requestId, uint256 offerId);
    event ProofAdded(uint256 indexed offerId, uint256 indexed blockNumber, bytes proof);
    event NewOffer(address indexed creator, address indexed provider, uint256 offerId);

    function startOffer(address providerAddress, uint256 dealLength, uint256 proofFrequency, uint256 bounty, uint256 collateral, address token, uint256 fileSize, string calldata cid, string calldata blake3) public payable returns(uint256)
    {

        _offerId++;
        _deals[_offerId].dealStartBlock = block.number;
        _deals[_offerId].dealLengthInBlocks = dealLength;
        _deals[_offerId].proofFrequencyInBlocks = proofFrequency;
        _deals[_offerId].price = bounty;
        _deals[_offerId].collateral = collateral;
        _deals[_offerId].erc20TokenDenomination = token;
        _deals[_offerId].fileSize = fileSize;
        _deals[_offerId].ipfsFileCID = cid;
        _deals[_offerId].blake3Checksum = blake3;
        _deals[_offerId].creatorCounterpart.partyAddress = msg.sender;
        _deals[_offerId].providerCounterpart.partyAddress = providerAddress;

        _deals[_offerId].creatorCounterpart.amount = bounty;
        
        _deals[_offerId].offerStatus = OfferStatus.OFFER_CREATED;
        _openOffers[msg.sender].push(_offerId);

        emit NewOffer(msg.sender, providerAddress, _offerId );
        return _offerId;
    }

     function getOffer(uint256 offerId) public view returns (uint256, uint256, uint256, uint256, uint256, address, string memory, uint256, string memory, address, address, uint8)
    {
        Deal storage store = _deals[offerId];
        return (
            store.dealStartBlock, 
            store.dealLengthInBlocks, 
            store.proofFrequencyInBlocks, 
            store.price,
            store.collateral,
            store.erc20TokenDenomination,
            store.ipfsFileCID,
            store.fileSize,
            store.blake3Checksum,
            store.creatorCounterpart.partyAddress, 
            store.providerCounterpart.partyAddress, 
            uint8(store.offerStatus));
    }
    
    function getDeal(uint256 offerID) public view returns (Deal memory) {
            return _deals[offerID];
        }
        
    function getDealStartBlock(uint256 offerID) public view returns (uint256) {
        return _deals[offerID].dealStartBlock;
    }
    function getDealStatus(uint256 _dealId) public view returns (uint8) {
        return uint8(_deals[_dealId].offerStatus);
    }
    function getDealLengthInBlocks(uint256 offerID) public view returns (uint256) {
        return _deals[offerID].dealLengthInBlocks;
    }
    function getProofFrequencyInBlocks(uint256 offerID) public view returns (uint256) {
        return _deals[offerID].proofFrequencyInBlocks;
    }
    function getPrice(uint256 offerID) public view returns (uint256) {
        return _deals[offerID].price;
    }
    function getCollateral(uint256 offerID) public view returns (uint256) {
        return _deals[offerID].collateral;
    }
    function getErc20TokenDenomination(uint256 offerID) public view returns (address) {
        return _deals[offerID].erc20TokenDenomination;
    }
    function getIpfsFileCid(uint256 offerID) public view returns (string memory) {
        return _deals[offerID].ipfsFileCID;
    }
    function getFileSize(uint256 offerID) public view returns (uint256) {
        return _deals[offerID].fileSize;
    }
    function getBlake3Checksum(uint256 offerID) public view returns (string memory) {
        return _deals[offerID].blake3Checksum;
    }
    function getProofBlock(uint256 offerID, uint256 windowNum) public view returns (uint256) {
        return _proofblocks[offerID][windowNum]; 
    }
    // get the block numbers of all proofs sent for a specific offer
    function getProofBlockNumbers(uint256 offerId) public view returns(uint256) {
        return _deals[offerId].dealLengthInBlocks;
    }

    // function that saves time of proof sending
    function saveProof(bytes calldata _proof, uint256 offerId, uint256 targetBlockNumber) public {
        require(_proof.length > 0, "No proof provided"); // check if proof is empty
        require(_deals[offerId].offerStatus == OfferStatus.OFFER_CREATED, "ERROR: OFFER_NOT_ACTIVE");
        require(targetBlockNumber < _deals[offerId].dealStartBlock + _deals[offerId].dealLengthInBlocks && block.number >= _deals[offerId].dealStartBlock, "Out of block timerange");
        require(block.number >= targetBlockNumber, "Proof cannot be sent in future");
        require(block.number <= targetBlockNumber + _deals[offerId].proofFrequencyInBlocks, "Saving proof outside of range");

        uint256 offset = targetBlockNumber - _deals[offerId].dealStartBlock;
        require(offset < _deals[offerId].dealLengthInBlocks, "Proof window is over"); // Potentially remove this revert as it is redundant with the above require.

        uint256 proofWindowNumber = offset / _deals[offerId].proofFrequencyInBlocks; // Proofs submit as entries within a range, denoted as the nth proofWindow.
        require(_proofblocks[offerId][proofWindowNumber] == 0, "Proof already submitted");
        

        _proofblocks[offerId][proofWindowNumber] = block.number;
        emit ProofAdded(offerId, _proofblocks[offerId][proofWindowNumber], _proof);
    }

    //  PART 2 j

    function requestVerification(string memory _jobId, string memory _blocknum, string memory _offerid) public returns (bytes32 requestId) {
        Chainlink.Request memory req = buildChainlinkRequest(stringToBytes32(_jobId), address(this), this.fulfill.selector);
        req.add("block_num", _blocknum); // proof blocknum
        req.add("offer_id", _offerid);
        return sendChainlinkRequest(req, fee);
    }

    /**
     * Receive the response in the form of uint256
     */
    function fulfill(bytes32 requestId, uint256 offerID, uint256 successCount, uint256 numWindows, uint16 status, string calldata result) public recordChainlinkFulfillment(requestId) {
        emit RequestVerification(requestId, offerID);
        responses[offerID] = ResponseData(offerID, successCount, numWindows, status, result);
    }

    /**
     * Allow withdraw of Link tokens from the contract
     */
    function withdrawLink() public onlyOwner {
        LinkTokenInterface link = LinkTokenInterface(chainlinkTokenAddress());
        require(link.transfer(msg.sender, link.balanceOf(address(this))), 'Unable to transfer');
    }

    function stringToBytes32(string memory source) private pure returns (bytes32 result) {
        bytes memory tempEmptyStringTest = bytes(source);
        if (tempEmptyStringTest.length == 0) {
            return 0x0;
        }
        assembly {
            // solhint-disable-line no-inline-assembly
            result := mload(add(source, 32))
        }
    }
}