// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {IERC20} from "../lib/openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";

contract DepositRegistry {
    address public admin; // Should be set to dstack container shared secret address
    uint256 public settlement_nonce;
    mapping(address => uint64[2][]) public deposit_registry; // [eth_amount, usdc_amount]
    IERC20 public constant WETH =
        IERC20(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2); // Mainnet WETH address
    address public constant HookTrampoline =
        0x0000000000000000000000000000000000000000; //TODO: Find correct contract address
    IERC20 public constant USDC =
        IERC20(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48); // Mainnet USDC address
    address public constant GPv2Settlement =
        0x0000000000000000000000000000000000000000; //TODO: Find correct contract address
    constructor() {
        admin = msg.sender;
    }

    function set_admin(address new_admin) external {
        require(msg.sender == admin, "Only the admin can set a new admin");
        admin = new_admin;
    }

    function deposit(
        address user,
        uint64 eth_amount,
        uint64 usdc_amount
    ) external {
        require(msg.sender == user, "Only the user can deposit"); // Transfer the specified amount of WETH from the specified address
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
    ) external view returns (uint64[2][] memory) {
        uint64[2][] memory deposits = deposit_registry[user];
        uint64[2][] memory amounts = new uint64[2][](deposits.length - _nonce);
        // iterates over every deposit after the nonce - getting every new deposit
        for (uint128 i = _nonce + 1; i < deposits.length; i++) {
            amounts[i - _nonce] = deposits[i];
        }
        return amounts;
    }

    // Pulls funds from the vault for a settlement order
    function pull_settlement_funds(
        uint64 eth_amount,
        uint64 usdc_amount,
        bytes memory signature,
        address to
    ) external {
        require(
            msg.sender == HookTrampoline,
            "Only the HookTrampoline contract can pull funds"
        );

        // Create a message hash that includes all relevant data
        bytes32 messageHash = keccak256(
            abi.encodePacked(
                eth_amount,
                usdc_amount,
                settlement_nonce,
                "pull_settlement_funds"
            )
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
            require(
                WETH.allowance(to, GPv2Settlement) >= eth_amount,
                "Insufficient taker allowance"
            );
            WETH.transfer(to, eth_amount);
        }
        if (usdc_amount > 0) {
            require(
                USDC.allowance(to, GPv2Settlement) >= usdc_amount,
                "Insufficient taker allowance"
            );
            USDC.transfer(to, usdc_amount);
        }
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
