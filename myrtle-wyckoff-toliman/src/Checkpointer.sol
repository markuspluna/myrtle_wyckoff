// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;
import {EfficientHashLib} from "../lib/solady/src/utils/EfficientHashLib.sol";
import {SignatureCheckerLib} from "../lib/solady/src/utils/SignatureCheckerLib.sol";

contract Checkpointer {
    address public admin; // Should be set to dstack app shared secret address
    uint256 public inventory_checkpoint_nonce;
    event SettlementOrders(string[] settlement_orders);
    /// @dev keccak256(
    ///     abi.encode(
    ///     keccak256(
    ///         "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
    ///     ),
    ///     keccak256(bytes(domain.name)),
    ///     keccak256(bytes(domain.version)),
    ///     domain.chainId,
    ///     domain.verifyingContract
    ///     )
    /// )
    bytes32 internal domainSeparator;

    // Vec of AES encoded inventories structured as (user: Address, eth_balance: i128, usdc_balance: i128, deposit nonce: u32, is_taker: u8)
    // In prod this should store multiple checkpoints and overwrite oldest with newest
    uint8[] public inventory_checkpoint;

    constructor() {
        admin = msg.sender;
    }
    function set_domain_separator(bytes32 domain_separator) external {
        if (msg.sender != admin) {
            revert("Only the admin can set the domain separator");
        }
        if (domainSeparator != 0) {
            revert("Domain separator already set");
        }
        domainSeparator = domain_separator;
    }

    function set_admin(address new_admin) external {
        require(msg.sender == admin, "Only the admin can set a new admin");
        admin = new_admin;
    }

    struct Checkpoint {
        uint256 nonce;
        uint8[] inventory_state;
        string[] settlement_orders;
    }

    // Register new blob containing encrypted inventory state
    function checkpoint(
        bytes calldata signature,
        Checkpoint calldata _checkpoint
    ) external {
        require(
            _checkpoint.nonce == inventory_checkpoint_nonce,
            "Nonce mismatch"
        );

        require(
            SignatureCheckerLib.isValidSignatureNow(
                admin,
                EfficientHashLib.hash(
                    abi.encodePacked(
                        "\x19\x01",
                        domainSeparator,
                        EfficientHashLib.hash(abi.encode(_checkpoint))
                    )
                ),
                signature
            ),
            "Invalid signature"
        );
        inventory_checkpoint_nonce++;
        inventory_checkpoint = _checkpoint.inventory_state;

        emit SettlementOrders(_checkpoint.settlement_orders);
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
