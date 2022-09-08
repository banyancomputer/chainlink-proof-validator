// SPDX-License-Identifier: MIT
pragma solidity ^0.8.7;

contract Test {

    struct Deal {
        string blake3_checksum;
    }
    mapping(uint256 => Deal) public deals; 

    // public setter function 
    function setDeal (string calldata _checksum) public {
        deals[0] = Deal({blake3_checksum: _checksum});
    }

    function setDealIndex (uint256 _offerId, string calldata _checksum) public {
        deals[_offerId] = Deal({blake3_checksum: _checksum});
    }

    function setDealStruct(uint256 _offerId, Deal calldata _deal) public {
        deals[_offerId] = _deal;
    }

    function setDealNothing() public {
        deals[0] = Deal({blake3_checksum: "dummmy"});
    }

    // public getter function 
    function getDeal (uint256 _offerId) public view returns (string memory) {
        return deals[_offerId].blake3_checksum;
    }
}