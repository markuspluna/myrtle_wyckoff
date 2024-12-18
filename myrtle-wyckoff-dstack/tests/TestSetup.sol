// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test} from "forge-std/Test.sol";
import {DepositRegistry} from "../src/DepositRegistry.sol";
import {Checkpointer} from "../src/Checkpointer.sol";
import {MockERC20} from "./mocks/MockERC20.sol";

contract TestSetup is Test {
    DepositRegistry public depositRegistry;
    Checkpointer public checkpointer;
    MockERC20 public weth;
    MockERC20 public usdc;
    
    address admin;
    uint256 adminKey;
    address user;
    
    function setUp() public {
        // Create deterministic addresses
        (admin, adminKey) = makeAddrAndKey("admin");
        user = makeAddr("user");

        // Deploy mock tokens at expected addresses
        vm.etch(address(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2), // WETH
               address(new MockERC20("Wrapped Ether", "WETH", 18)).code);
        vm.etch(address(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48), // USDC
               address(new MockERC20("USD Coin", "USDC", 6)).code);
        
        weth = MockERC20(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
        usdc = MockERC20(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48);

        vm.startPrank(admin);
        depositRegistry = new DepositRegistry();
        checkpointer = new Checkpointer();
        vm.stopPrank();

        // Setup initial balances
        deal(address(weth), user, 100 ether);
        deal(address(usdc), user, 100_000 * 1e6);
    }
} 