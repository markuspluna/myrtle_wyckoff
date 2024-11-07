// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {DepositRegistry} from "../src/DepositRegistry.sol";
import {ERC20} from "../lib/solady/src/tokens/ERC20.sol";
import {MockERC20} from "../lib/forge-std/src/mocks/MockERC20.sol";

contract DepositRegistryTest is Test {
    DepositRegistry public depositRegistry;
    address public admin;
    uint256 adminKey;
    address public user;
    bytes32 domain_hash;
    ERC20 public constant WETH =
        ERC20(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2);
    ERC20 public constant USDC =
        ERC20(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48);
    address internal constant HookTrampoline =
        0x01DcB88678aedD0C4cC9552B20F4718550250574;
    address internal constant GPv2Settlement =
        0x9008D19f58AAbD9eD0D60971565AA8510560ab41;

    function setUp() public {
        (admin, adminKey) = makeAddrAndKey("admin");
        user = makeAddr("user");

        // Deploy mock tokens at the expected addresses
        vm.etch(address(WETH), address(new MockERC20()).code);
        vm.etch(address(USDC), address(new MockERC20()).code);

        // Get references to the mocks at the expected addresses
        MockERC20 weth = MockERC20(address(WETH));
        MockERC20 usdc = MockERC20(address(USDC));
        weth.initialize("Wrapped Ether", "WETH", 18);
        usdc.initialize("USD Coin", "USDC", 6);

        vm.startPrank(admin);
        depositRegistry = new DepositRegistry();

        domain_hash = keccak256(
            abi.encode(
                keccak256(
                    "EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)"
                ),
                keccak256(bytes("MyrtleWyckoff")),
                keccak256(bytes("1")),
                block.chainid,
                address(depositRegistry)
            )
        );
        depositRegistry.set_domain_separator(domain_hash);
        vm.stopPrank();

        // Deal tokens to user
        deal(address(WETH), user, 100 ether);
        deal(address(USDC), user, 100_000 * 1e6);

        vm.startPrank(user);
        weth.approve(address(depositRegistry), type(uint256).max);
        usdc.approve(address(depositRegistry), type(uint256).max);
        vm.stopPrank();
    }

    function test_InitialState() public view {
        assertEq(depositRegistry.admin(), admin);
        assertEq(depositRegistry.settlement_nonce(), 0);
    }

    function test_SetAdmin() public {
        address newAdmin = makeAddr("newAdmin");
        vm.prank(admin);
        depositRegistry.set_admin(newAdmin);
        assertEq(depositRegistry.admin(), newAdmin);
    }

    function test_SetAdmin_OnlyAdmin() public {
        address newAdmin = makeAddr("newAdmin");
        vm.prank(user);
        vm.expectRevert("Only the admin can set a new admin");
        depositRegistry.set_admin(newAdmin);
    }

    function test_Deposit() public {
        uint256 ethAmount = 1 ether;
        uint256 usdcAmount = 1000 * 1e6;

        vm.prank(user);
        depositRegistry.deposit(user, ethAmount, usdcAmount);

        uint256[2][] memory deposits = depositRegistry.get_deposits(0, user);
        assertEq(deposits[0][0], ethAmount);
        assertEq(deposits[0][1], usdcAmount);
    }

    function test_Deposit_OnlyUser() public {
        vm.prank(admin);
        vm.expectRevert("Only the user can deposit");
        depositRegistry.deposit(user, 1 ether, 1000 * 1e6);
    }

    function test_GetDeposits() public {
        // Make multiple deposits
        vm.startPrank(user);
        depositRegistry.deposit(user, 1 ether, 1000 * 1e6);
        depositRegistry.deposit(user, 2 ether, 2000 * 1e6);
        depositRegistry.deposit(user, 3 ether, 3000 * 1e6);
        vm.stopPrank();

        // Get deposits after nonce 1
        uint256[2][] memory deposits = depositRegistry.get_deposits(0, user);
        assertEq(deposits.length, 3);
        assertEq(deposits[0][0], 1 ether);
        assertEq(deposits[0][1], 1000 * 1e6);
        assertEq(deposits[1][0], 2 ether);
        assertEq(deposits[1][1], 2000 * 1e6);
        assertEq(deposits[2][0], 3 ether);
        assertEq(deposits[2][1], 3000 * 1e6);
    }

    function test_GetDeposits_OutOfBounds() public {
        vm.expectRevert("Nonce is out of bounds");
        depositRegistry.get_deposits(1, user);
    }

    function test_PullSettlementFunds() public {
        // Create settlement order
        DepositRegistry.Order memory order = DepositRegistry.Order({
            ethAmount: 1 ether,
            usdcAmount: 1000 * 1e6,
            isBid: true,
            nonce: 0
        });

        // Create signature
        bytes32 encodedOrder = keccak256(abi.encode(order));
        bytes32 message = keccak256(
            abi.encodePacked("\x19\x01", domain_hash, encodedOrder)
        );
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(adminKey, message);
        bytes memory signature = abi.encodePacked(r, s, v);

        // Call pull_settlement_funds as HookTrampoline
        vm.prank(HookTrampoline);
        depositRegistry.pull_settlement_funds(order, signature);

        // Verify state changes
        assertEq(depositRegistry.settlement_nonce(), 1);
        assertEq(
            WETH.allowance(address(depositRegistry), GPv2Settlement),
            1 ether
        );
        assertEq(USDC.allowance(address(depositRegistry), GPv2Settlement), 0);
    }

    function test_PullSettlementFunds_OnlyHookTrampoline() public {
        DepositRegistry.Order memory order = DepositRegistry.Order({
            ethAmount: 1 ether,
            usdcAmount: 1000 * 1e6,
            isBid: true,
            nonce: 0
        });

        vm.prank(user);
        vm.expectRevert("Only the HookTrampoline contract can pull funds");
        depositRegistry.pull_settlement_funds(order, new bytes(0));
    }
}
