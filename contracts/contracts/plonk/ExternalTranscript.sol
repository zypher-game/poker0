// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

library ExternalTranscript {
    function load(uint256 loc) internal pure {
        assembly {
            mstore(loc, 1) // the length
            mstore(add(loc, 0x20), 0x506c6f6e6b20706f6b65722050726f6f66) 
        }
    }
}
