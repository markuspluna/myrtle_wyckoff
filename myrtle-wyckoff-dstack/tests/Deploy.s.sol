// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script} from "forge-std/Script.sol";
import {DepositRegistry} from "../src/DepositRegistry.sol";
import {Checkpointer} from "../src/Checkpointer.sol";

contract Deploy is Script {
    function run() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        DepositRegistry depositRegistry = new DepositRegistry();
        Checkpointer checkpointer = new Checkpointer();

        vm.stopBroadcast();
    }
} 