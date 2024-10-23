// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.28;

import {Script, console} from "forge-std/Script.sol";
import {MyrtleWyckoff} from "../src/MyrtleWyckoff.sol";

contract MyrtleWyckoffScript is Script {
    MyrtleWyckoff public myrtleWyckoff;

    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        myrtleWyckoff = new MyrtleWyckoff();

        vm.stopBroadcast();
    }
}
