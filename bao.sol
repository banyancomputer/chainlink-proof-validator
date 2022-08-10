// SPDX-License-Identifier: MIT
pragma solidity >= 0.8.9;

contract bao_log {

    event ProofAdded(uint256 indexed offerId, uint256 indexed blockNumber, bytes proof);

    function save_proof () public {
        uint256 offerId = 1;
        bytes memory _proof = "Jonah and zev to the rescue weeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"; 
        emit ProofAdded(offerId, block.number, _proof);
    }
}