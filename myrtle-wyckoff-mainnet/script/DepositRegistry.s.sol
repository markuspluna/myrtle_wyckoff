// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {DepositRegistry} from "../src/DepositRegistry.sol";

contract DepositRegistryScript is Script {
    DepositRegistry public depositRegistry;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        depositRegistry = new DepositRegistry();

        vm.stopBroadcast();
    }
}
