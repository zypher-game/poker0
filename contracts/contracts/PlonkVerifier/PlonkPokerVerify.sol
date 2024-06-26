// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.20;

import "./verifier/Verifier.sol";
import "./VerifierKey.sol";
import "./ExternalTranscript.sol";

contract PlonkPokerVerifier is Verifier {
    address _extraVk1;
    address _extraVk2;

    constructor(address _vk1, address _vk2) {
        _extraVk1 = _vk1;
        _extraVk2 = _vk2;
    }

    function verify(bytes calldata _proof, uint256[] calldata _publicInputs) public view returns (bool) {
        VerifierKey.load(CM_Q0_X_LOC, PI_POLY_RELATED_LOC);
        ExternalTranscript.load(EXTERNAL_TRANSCRIPT_LENGTH_LOC);

        // The scalar field of BN254.
        uint256 r = 21888242871839275222246405745257275088548364400416034343698204186575808495617;

        // Load the proof.
        assembly {
            let data_ptr := add(calldataload(0x04), 0x24)
            mstore(CM_W0_X_LOC, mod(calldataload(add(data_ptr, 0x00)), r))
            mstore(CM_W0_Y_LOC, mod(calldataload(add(data_ptr, 0x20)), r))
            mstore(CM_W1_X_LOC, mod(calldataload(add(data_ptr, 0x40)), r))
            mstore(CM_W1_Y_LOC, mod(calldataload(add(data_ptr, 0x60)), r))
            mstore(CM_W2_X_LOC, mod(calldataload(add(data_ptr, 0x80)), r))
            mstore(CM_W2_Y_LOC, mod(calldataload(add(data_ptr, 0xa0)), r))
            mstore(CM_W3_X_LOC, mod(calldataload(add(data_ptr, 0xc0)), r))
            mstore(CM_W3_Y_LOC, mod(calldataload(add(data_ptr, 0xe0)), r))
            mstore(CM_W4_X_LOC, mod(calldataload(add(data_ptr, 0x100)), r))
            mstore(CM_W4_Y_LOC, mod(calldataload(add(data_ptr, 0x120)), r))
            mstore(CM_T0_X_LOC, mod(calldataload(add(data_ptr, 0x140)), r))
            mstore(CM_T0_Y_LOC, mod(calldataload(add(data_ptr, 0x160)), r))
            mstore(CM_T1_X_LOC, mod(calldataload(add(data_ptr, 0x180)), r))
            mstore(CM_T1_Y_LOC, mod(calldataload(add(data_ptr, 0x1a0)), r))
            mstore(CM_T2_X_LOC, mod(calldataload(add(data_ptr, 0x1c0)), r))
            mstore(CM_T2_Y_LOC, mod(calldataload(add(data_ptr, 0x1e0)), r))
            mstore(CM_T3_X_LOC, mod(calldataload(add(data_ptr, 0x200)), r))
            mstore(CM_T3_Y_LOC, mod(calldataload(add(data_ptr, 0x220)), r))
            mstore(CM_T4_X_LOC, mod(calldataload(add(data_ptr, 0x240)), r))
            mstore(CM_T4_Y_LOC, mod(calldataload(add(data_ptr, 0x260)), r))
            mstore(CM_Z_X_LOC, mod(calldataload(add(data_ptr, 0x280)), r))
            mstore(CM_Z_Y_LOC, mod(calldataload(add(data_ptr, 0x2a0)), r))
            mstore(PRK_3_EVAL_ZAETA_LOC, mod(calldataload(add(data_ptr, 0x2c0)), r))
            mstore(PRK_4_EVAL_ZAETA_LOC, mod(calldataload(add(data_ptr, 0x2e0)), r))
            mstore(W_POLY_EVAL_ZAETA_0_LOC, mod(calldataload(add(data_ptr, 0x300)), r))
            mstore(W_POLY_EVAL_ZAETA_1_LOC, mod(calldataload(add(data_ptr, 0x320)), r))
            mstore(W_POLY_EVAL_ZAETA_2_LOC, mod(calldataload(add(data_ptr, 0x340)), r))
            mstore(W_POLY_EVAL_ZAETA_3_LOC, mod(calldataload(add(data_ptr, 0x360)), r))
            mstore(W_POLY_EVAL_ZAETA_4_LOC, mod(calldataload(add(data_ptr, 0x380)), r))
            mstore(W_POLY_EVAL_ZAETA_OMEGA_0_LOC, mod(calldataload(add(data_ptr, 0x3a0)), r))
            mstore(W_POLY_EVAL_ZAETA_OMEGA_1_LOC, mod(calldataload(add(data_ptr, 0x3c0)), r))
            mstore(W_POLY_EVAL_ZAETA_OMEGA_2_LOC, mod(calldataload(add(data_ptr, 0x3e0)), r))
            mstore(Z_EVAL_ZETA_OMEGA_LOC, mod(calldataload(add(data_ptr, 0x400)), r))
            mstore(S_POLY_EVAL_ZAETA_0_LOC, mod(calldataload(add(data_ptr, 0x420)), r))
            mstore(S_POLY_EVAL_ZAETA_1_LOC, mod(calldataload(add(data_ptr, 0x440)), r))
            mstore(S_POLY_EVAL_ZAETA_2_LOC, mod(calldataload(add(data_ptr, 0x460)), r))
            mstore(S_POLY_EVAL_ZAETA_3_LOC, mod(calldataload(add(data_ptr, 0x480)), r))
            mstore(OPENING_ZETA_X_LOC, mod(calldataload(add(data_ptr, 0x4a0)), r))
            mstore(OPENING_ZETA_Y_LOC, mod(calldataload(add(data_ptr, 0x4c0)), r))
            mstore(OPENING_ZETA_OMEGA_X_LOC, mod(calldataload(add(data_ptr, 0x4e0)), r))
            mstore(OPENING_ZETA_OMEGA_Y_LOC, mod(calldataload(add(data_ptr, 0x500)), r))
        }

        // Load the public inputs.
        assembly {
            let pi_ptr := add(calldataload(0x24), 0x04)
            let pi_length := calldataload(add(pi_ptr, 0x00))
            let store_ptr := add(PI_POLY_RELATED_LOC, 0x20)

            for {
                let i := 0
            } lt(i, pi_length) {
                i := add(i, 1)
            } {
                mstore(add(store_ptr, mul(i, 0x20)), calldataload(add(add(pi_ptr, 0x20), mul(i, 0x20))))
            }
        }

        return verify_proof(_extraVk1, _extraVk2);
    }
}
