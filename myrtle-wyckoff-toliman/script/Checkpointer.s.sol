// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {Checkpointer} from "../src/Checkpointer.sol";

contract CheckpointerScript is Script {
    Checkpointer public checkpointer;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        checkpointer = new Checkpointer();

        vm.stopBroadcast();
    }
}
