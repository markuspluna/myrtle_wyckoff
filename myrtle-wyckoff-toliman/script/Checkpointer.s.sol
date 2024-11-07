// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";
import {Checkpointer} from "../src/Checkpointer.sol";

contract CheckpointerScript is Script {
    Checkpointer public checkpointer;

    function setUp() public {
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        vm.startBroadcast(deployerPrivateKey);

        checkpointer = new Checkpointer();

        bytes32 domain_hash = keccak256(
            abi.encode(
                keccak256(
                    "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
                ),
                keccak256(bytes("1")),
                keccak256(bytes("1")),
                1,
                address(checkpointer)
            )
        );
        checkpointer.set_domain_separator(domain_hash);
        vm.stopBroadcast();
    }

    function run() public {
        vm.startBroadcast();

        checkpointer = new Checkpointer();

        vm.stopBroadcast();
    }
}
