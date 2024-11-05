// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {Checkpointer} from "../src/Checkpointer.sol";

contract CheckpointerTest is Test {
    Checkpointer public checkpointer;
    address admin;
    uint256 adminKey;

    function setUp() public {
        // Create a deterministic admin address for testing
        (admin, adminKey) = makeAddrAndKey("admin");
        vm.prank(admin);
        checkpointer = new Checkpointer();
    }

    function test_InitialState() public {
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
        bytes32 domainSeparator = checkpointer._DOMAIN_TYPEHASH();
        bytes32 message = keccak256(
            abi.encodePacked("\x19\x01", domainSeparator, encodedCheckpoint)
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
