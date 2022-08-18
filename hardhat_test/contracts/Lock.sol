// SPDX-License-Identifier: MIT
pragma solidity ^0.8.9;

contract Lock {

    uint256 private offerId;

    mapping(uint256 => onChainDealInfo) internal deals;

    event ProofAdded(uint256 indexed offerId, uint256 indexed blockNumber, bytes proof);
    struct onChainDealInfo {
        uint256 offerId;
        uint256 deal_start_block;
        uint256 deal_length_in_blocks;
        uint256 proof_frequency_in_blocks;
        uint256 price;
        uint256 collateral;
        address erc20_token_denomination;
        uint256 ipfs_file_cid; 
        uint256 file_size;
        uint256 blake3_checksum;
    }

    function createOffer(
        uint256 _offerId,
        uint256 _deal_start_block,
        uint256 _deal_length_in_blocks,
        uint256 _proof_frequency_in_blocks,
        uint256 _price,
        uint256 _collateral,
        address _erc20_token_denomination,
        uint256 _ipfs_file_cid,
        uint256 _file_size,
        uint256 _blake3_checksum)
        public{
            deals[_offerId].offerId = _offerId;
            deals[_offerId].deal_start_block = _deal_start_block;
            deals[_offerId].deal_length_in_blocks = _deal_length_in_blocks;
            deals[_offerId].proof_frequency_in_blocks = _proof_frequency_in_blocks;
            deals[_offerId].price = _price;
            deals[_offerId].collateral = _collateral;
            deals[_offerId].erc20_token_denomination = _erc20_token_denomination;
            deals[_offerId].ipfs_file_cid = _ipfs_file_cid;
            deals[_offerId].file_size = _file_size;
            deals[_offerId].blake3_checksum = _blake3_checksum;
        }

    function getOffer(uint256 _offerId) public view returns (onChainDealInfo memory) {
        return deals[_offerId];
    }

     function save_proof (bytes calldata _proof) public {
        emit ProofAdded(613, block.number, _proof);
    }
}
