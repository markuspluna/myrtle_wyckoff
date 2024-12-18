// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script} from "forge-std/Script.sol";
import {DepositRegistry} from "../src/DepositRegistry.sol";
import {MockERC20} from "../test/mocks/MockERC20.sol";

contract SetupDeposits is Script {
    function run(address user) public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        // Get contract references
        DepositRegistry depositRegistry = DepositRegistry(0x...); // Get from deployment
        MockERC20 weth = MockERC20(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
        MockERC20 usdc = MockERC20(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48);

        // Setup test deposits
        weth.mint(user, 10 ether);
        usdc.mint(user, 15_000 * 1e6);

        vm.stopPrank();
        vm.startPrank(user);
        
        weth.approve(address(depositRegistry), type(uint256).max);
        usdc.approve(address(depositRegistry), type(uint256).max);
        
        depositRegistry.deposit(
            user,
            1 ether,
            1000 * 1e6
        );

        vm.stopPrank();
    }
} 