// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {ERC20} from "../lib/solady/src/tokens/ERC20.sol";
import {EfficientHashLib} from "../lib/solady/src/utils/EfficientHashLib.sol";
import {SignatureCheckerLib} from "../lib/solady/src/utils/SignatureCheckerLib.sol";

contract DepositRegistry {
    address public admin; // Should be set to dstack container shared secret address
    uint256 public settlement_nonce;
    mapping(address => uint256[2][]) public deposit_registry; // [eth_amount, usdc_amount]
    ERC20 public constant WETH =
        ERC20(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2); // Mainnet WETH address
    address public constant HookTrampoline =
        0x0000000000000000000000000000000000000000; //TODO: Find correct contract address
    ERC20 public constant USDC =
        ERC20(0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48); // Mainnet USDC address
    address public constant GPv2Settlement =
        0x0000000000000000000000000000000000000000; //TODO: Find correct contract address
    /// @dev `keccak256("EIP712Domain(string name,string version,uint256 chainId,address verifyingContract)")`.
    bytes32 internal constant _DOMAIN_TYPEHASH =
        0x8b73c3c69bb8fe3d512ecc4cf759cc79239f7b179b0ffacaa9a75d522b39400f; //TODO: update with correct domain typehash
    constructor() {
        admin = msg.sender;
    }

    function set_admin(address new_admin) external {
        require(msg.sender == admin, "Only the admin can set a new admin");
        admin = new_admin;
    }

    function deposit(
        address user,
        uint256 eth_amount,
        uint256 usdc_amount
    ) external {
        require(msg.sender == user, "Only the user can deposit"); // Transfer the specified amount of WETH from the specified address
        if (eth_amount > 0) {
            WETH.transferFrom(user, address(this), eth_amount);
        }
        if (usdc_amount > 0) {
            USDC.transferFrom(user, address(this), usdc_amount);
        }
        deposit_registry[user].push([eth_amount, usdc_amount]);
    }

    function get_deposits(
        uint32 _nonce,
        address user
    ) external view returns (uint256[2][] memory) {
        uint256[2][] memory deposits = deposit_registry[user];
        uint256[2][] memory amounts = new uint256[2][](
            deposits.length - _nonce
        );
        // iterates over every deposit after the nonce - getting every new deposit
        for (uint32 i = _nonce + 1; i < deposits.length; i++) {
            amounts[i - _nonce] = deposits[i];
        }
        return amounts;
    }

    /// @notice Represents a settlement order with amounts and direction
    /// @param ethAmount The amount of ETH in the order
    /// @param usdcAmount The amount of USDC in the order
    /// @param isBid True if this is a bid order, false if ask
    /// @param nonce The settlement nonce for replay protection
    struct Order {
        uint256 ethAmount;
        uint256 usdcAmount;
        bool isBid;
        uint256 nonce;
    }

    // Approves a pull of funds for a settlement order
    function pull_settlement_funds(
        Order calldata settlement_order,
        bytes calldata signature
    ) external {
        // Only the HookTrampoline contract can pull funds
        require(
            msg.sender == HookTrampoline,
            "Only the HookTrampoline contract can pull funds"
        );
        // Nonce must match the current nonce
        require(settlement_order.nonce == settlement_nonce, "Nonce mismatch");
        // Signature must be from the admin (dstack app shared secret)
        require(
            SignatureCheckerLib.isValidSignatureNowCalldata(
                admin,
                EfficientHashLib.hash(
                    abi.encodePacked(
                        "\x19\x01",
                        _DOMAIN_TYPEHASH,
                        EfficientHashLib.hash(abi.encode(settlement_order))
                    )
                ),
                signature
            ),
            "Invalid signature"
        );

        // Increment nonce for replay protection
        settlement_nonce++;
        // Approve the GPv2Settlement contract to pull the specified amount of WETH and USDC
        if (settlement_order.isBid) {
            WETH.approve(GPv2Settlement, settlement_order.ethAmount);
        } else {
            USDC.approve(GPv2Settlement, settlement_order.usdcAmount);
        }
    }

    // EIP-1271 signature validation
    function isValidSignature(
        bytes32 _hash,
        bytes memory _signature
    ) external view returns (bytes4) {
        // Validate signature is from admin
        bool isValid = SignatureCheckerLib.isValidSignatureNow(
            admin,
            EfficientHashLib.hash(
                abi.encodePacked("\x19\x01", _DOMAIN_TYPEHASH, _hash)
            ),
            _signature
        );
        if (isValid) {
            // Return EIP-1271 magic value if valid
            return 0x1626ba7e;
        } else {
            // Return invalid magic value if invalid
            return 0xffffffff;
        }
    }
}
