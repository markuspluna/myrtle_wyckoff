// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Checkpointer} from "../src/Checkpointer.sol";

contract CheckpointerTest is Test {
    Checkpointer public checkpointer;
    address admin;
    uint256 adminKey;
    bytes32 domain_hash;

    function setUp() public {
        // Create a deterministic admin address for testing
        (admin, adminKey) = makeAddrAndKey("admin");
        vm.startBroadcast(adminKey);
        checkpointer = new Checkpointer();
        // Set domain hash
        domain_hash = keccak256(
            abi.encode(
                keccak256(
                    "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
                ),
                keccak256(bytes("MyrtleWyckoff")),
                keccak256(bytes("1")),
                33626250,
                address(checkpointer)
            )
        );
        checkpointer.set_domain_separator(domain_hash);
        vm.stopBroadcast();
    }

    function test_InitialState() public view {
        assertEq(checkpointer.admin(), admin);
        assertEq(checkpointer.inventory_checkpoint_nonce(), 0);
    }

    function test_SetAdmin() public {
        address newAdmin = makeAddr("newAdmin");

        // Should fail when called by non-admin
        vm.prank(newAdmin);
        vm.expectRevert("Only the admin can set a new admin");
        checkpointer.set_admin(newAdmin);

        // Should succeed when called by admin
        vm.prank(admin);
        checkpointer.set_admin(newAdmin);
        assertEq(checkpointer.admin(), newAdmin);
    }

    function test_Checkpoint() public {
        uint8[] memory inventoryState = new uint8[](2);
        inventoryState[0] = 1;
        inventoryState[1] = 2;

        string[] memory settlementOrders = new string[](1);
        settlementOrders[0] = "order1";

        Checkpointer.Checkpoint memory checkpoint = Checkpointer.Checkpoint({
            nonce: 0,
            inventory_state: inventoryState,
            settlement_orders: settlementOrders
        });

        // Create signature
        bytes32 encodedCheckpoint = keccak256(abi.encode(checkpoint));
        bytes32 message = keccak256(
            abi.encodePacked("\x19\x01", domain_hash, encodedCheckpoint)
        );
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(adminKey, message);
        bytes memory signature = abi.encodePacked(r, s, v);

        // Test checkpoint submission
        vm.expectEmit(true, true, true, true);
        emit Checkpointer.SettlementOrders(settlementOrders);

        checkpointer.checkpoint(signature, checkpoint);

        // Verify state changes
        assertEq(checkpointer.inventory_checkpoint_nonce(), 1);
        assertEq(checkpointer.inventory_checkpoint(0), 1);
        assertEq(checkpointer.inventory_checkpoint(1), 2);

        uint8[] memory inventoryState_2 = new uint8[](2);
        inventoryState_2[0] = 3;
        inventoryState_2[1] = 4;

        string[] memory settlementOrders_2 = new string[](1);
        settlementOrders_2[0] = "order2";

        Checkpointer.Checkpoint memory checkpoint_2 = Checkpointer.Checkpoint({
            nonce: 1,
            inventory_state: inventoryState_2,
            settlement_orders: settlementOrders_2
        });

        // Create signature
        bytes32 encodedCheckpoint_2 = keccak256(abi.encode(checkpoint_2));
        bytes32 message_2 = keccak256(
            abi.encodePacked("\x19\x01", domain_hash, encodedCheckpoint_2)
        );
        (uint8 v_2, bytes32 r_2, bytes32 s_2) = vm.sign(adminKey, message_2);
        bytes memory signature_2 = abi.encodePacked(r_2, s_2, v_2);

        // Test checkpoint submission
        vm.expectEmit(true, true, true, true);
        emit Checkpointer.SettlementOrders(settlementOrders_2);

        checkpointer.checkpoint(signature_2, checkpoint_2);

        // Verify state changes
        assertEq(checkpointer.inventory_checkpoint_nonce(), 2);
        assertEq(checkpointer.inventory_checkpoint(0), 3);
        assertEq(checkpointer.inventory_checkpoint(1), 4);
    }

    function testFail_CheckpointInvalidNonce() public {
        uint8[] memory inventoryState = new uint8[](1);
        string[] memory settlementOrders = new string[](0);

        Checkpointer.Checkpoint memory checkpoint = Checkpointer.Checkpoint({
            nonce: 1, // Invalid nonce
            inventory_state: inventoryState,
            settlement_orders: settlementOrders
        });

        bytes memory signature = new bytes(65);
        checkpointer.checkpoint(signature, checkpoint);
    }
}
