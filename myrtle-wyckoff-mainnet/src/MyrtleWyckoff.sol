// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

import {IERC20} from "../lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

contract MyrtleWyckoff {
    address public admin; // Should be set to dstack container shared secret address
    uint256 public settlement_nonce;
    mapping(uint256 => bytes32) public blob_registry;
    mapping(address => uint64[2][]) public deposit_registry; // [eth_amount, usdc_amount]
    IERC20 public constant WETH =
        IERC20(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2); // Mainnet WETH address
    IERC20 public constant USDC =
        IERC20(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48); // Mainnet USDC address
    address public constant HookTrampoline =
        0x0000000000000000000000000000000000000000; //TODO: Find correct contract address

    constructor() {
        admin = msg.sender;
    }

    function set_admin(address new_admin) public {
        require(msg.sender == admin, "Only the admin can set a new admin");
        admin = new_admin;
    }

    function deposit(
        address user,
        uint64 eth_amount,
        uint64 usdc_amount
    ) public {
        // Transfer the specified amount of WETH from the specified address
        if (eth_amount > 0) {
            WETH.transferFrom(user, address(this), eth_amount);
        }
        if (usdc_amount > 0) {
            USDC.transferFrom(user, address(this), usdc_amount);
        }
        deposit_registry[user].push([eth_amount, usdc_amount]);
    }

    function get_deposits(
        uint128 _nonce,
        address user
    ) public view returns (uint64[2][] memory) {
        uint64[2][] memory deposits = deposit_registry[user];
        uint64[2][] memory amounts = new uint64[2][](deposits.length - _nonce);
        // iterates over every deposit after the nonce - getting every new deposit
        for (uint128 i = _nonce + 1; i < deposits.length; i++) {
            amounts[i - _nonce] = deposits[i];
        }
        return amounts;
    }

    // Pulls funds from the vault for a settlement order
    // TODO: it's insecure to use the shared dstack key as admin here
    // probably fine for a poc but an attacker could register as a dstack
    // node and write their own program that creates signatures for malicious
    // settlement fund pulls
    function pull_settlement_funds(
        uint64 eth_amount,
        uint64 usdc_amount,
        bytes memory signature,
        address to //may hard code this as the cowswap settlement contract
    ) public {
        require(
            msg.sender == HookTrampoline,
            "Only the HookTrampoline contract can pull funds"
        );

        // Create a message hash that includes all relevant data
        bytes32 messageHash = keccak256(
            abi.encodePacked(eth_amount, usdc_amount, to, settlement_nonce)
        );

        // Prefix the hash with the Ethereum Signed Message prefix
        bytes32 prefixedHash = keccak256(
            abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash)
        );

        require(
            validateSignature(signature, prefixedHash, admin),
            "Signature is not from the admin"
        );

        // Increment nonce for replay protection
        settlement_nonce++;
        // Transfer weth to the specified address
        if (eth_amount > 0) {
            WETH.transfer(to, eth_amount);
        }
        if (usdc_amount > 0) {
            USDC.transfer(to, usdc_amount);
        }
    }

    // Register new blob containing encrypted inventory state
    function register_blob(bytes memory signature, uint256 blob_nonce) public {
        // Create a message hash that includes all relevant data
        bytes32 messageHash = keccak256(
            abi.encodePacked("register_blob", blob_nonce)
        );

        // Prefix the hash with the Ethereum Signed Message prefix
        bytes32 prefixedHash = keccak256(
            abi.encodePacked("\x19Ethereum Signed Message:\n32", messageHash)
        );

        require(
            validateSignature(signature, prefixedHash, admin),
            "Signature is not from the admin"
        );
        if (blob_registry[blob_nonce].length != 0) {
            revert("Blob nonce already used");
        }
        // Only 1 blob per inventory state checkpoint, limits inventory size so a different implementation would be needed in prod
        blob_registry[blob_nonce] = blobhash(0);
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
