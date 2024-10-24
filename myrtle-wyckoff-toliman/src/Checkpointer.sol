// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

contract Checkpointer {
    address public admin; // Should be set to dstack app shared secret address
    uint256 public inventory_checkpoint_nonce;
    event SettlementOrders(string[] settlement_orders);

    // Vec of AES encoded inventories structured as (user: Address, eth_balance: i128, usdc_balance: i128, deposit nonce: u32, is_taker: u8)
    // In prod this should store multiple checkpoints and overwrite oldest with newest
    uint8[] public inventory_checkpoint;

    constructor() {
        admin = msg.sender;
    }

    function set_admin(address new_admin) external {
        require(msg.sender == admin, "Only the admin can set a new admin");
        admin = new_admin;
    }

    // Register new blob containing encrypted inventory state
    function checkpoint(
        bytes calldata signature,
        uint256 nonce,
        uint8[] calldata inventory_state,
        string[] calldata settlement_orders
    ) external {
        // Create a message hash that includes all relevant data
        bytes32 messageHash = keccak256(
            abi.encodePacked(nonce, inventory_state)
        );

        // Prefix the hash with the Ethereum Signed Message prefix
        bytes32 prefixedHash = keccak256(
            abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash)
        );

        require(
            validateSignature(signature, prefixedHash, admin),
            "Signature is not from the admin"
        );
        require(nonce == inventory_checkpoint_nonce, "Nonce mismatch");
        inventory_checkpoint_nonce++;
        inventory_checkpoint = inventory_state;

        emit SettlementOrders(settlement_orders);
    }

    function validateSignature(
        bytes memory signature,
        bytes32 messageHash,
        address expectedSigner
    ) internal pure returns (bool) {
        require(signature.length == 65, "Invalid signature length");

        bytes32 r;
        bytes32 s;
        uint8 v;

        assembly {
            r := mload(add(signature, 32))
            s := mload(add(signature, 64))
            v := byte(0, mload(add(signature, 96)))
        }

        if (v < 27) {
            v += 27;
        }

        address recoveredAddress = ecrecover(messageHash, v, r, s);
        return recoveredAddress == expectedSigner;
    }
}
