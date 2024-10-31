// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

interface IDepositRegistry {
    struct Order {
        uint256 ethAmount;
        uint256 usdcAmount;
        bool isBid;
        uint256 nonce;
    }

    function GPv2Settlement() external view returns (address);
    function HookTrampoline() external view returns (address);
    function USDC() external view returns (address);
    function WETH() external view returns (address);
    function admin() external view returns (address);
    function deposit(
        address user,
        uint256 ethAmount,
        uint256 usdcAmount
    ) external;
    function deposit_registry(
        address user,
        uint256 nonce,
        uint256 index
    ) external view returns (uint256);
    function get_deposits(
        uint32 nonce,
        address user
    ) external view returns (uint256[2][] memory);
    function isValidSignature(
        bytes32 hash,
        bytes memory signature
    ) external view returns (bytes4);
    function pull_settlement_funds(
        Order calldata order,
        bytes calldata signature
    ) external;
    function set_admin(address newAdmin) external;
    function settlement_nonce() external view returns (uint256);
}
