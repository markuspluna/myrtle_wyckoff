// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Checkpointer} from "../src/Checkpointer.sol";

contract CheckpointerTest is Test {
    Checkpointer public checkpointer;

    function setUp() public {
        checkpointer = new Checkpointer();
    }
}
