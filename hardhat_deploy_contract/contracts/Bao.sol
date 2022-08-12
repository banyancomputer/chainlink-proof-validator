// SPDX-License-Identifier: MIT
pragma solidity >= 0.8.9;

contract Bao {

    event ProofAdded(uint256 indexed offerId, uint256 indexed blockNumber, bytes proof);

    function save_proof (bytes calldata _proof) public {
        uint256 offerId = 613;
        emit ProofAdded(offerId, block.number, _proof);
    }
}
