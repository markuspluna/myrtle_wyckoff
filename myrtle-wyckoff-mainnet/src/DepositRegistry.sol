// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

//import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract DepositRegistry {
    address public admin;
    uint64 public deposit_amount;
    mapping(address => uint64[]) public deposit_registry;
    // IERC20 public constant WETH =
    //     IERC20(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2); // Mainnet WETH address

    constructor() {
        admin = msg.sender;
    }

    // function deposit(address user, uint64 amount) public {
    //     require(
    //         msg.sender != address(0),
    //         "Cannot deposit from the zero address"
    //     );
    //     // Transfer the specified amount of WETH from the specified address
    //     IERC20(0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2).transferFrom(
    //         msg.sender,
    //         address(this),
    //         amount
    //     );
    // }

    // function get_deposits(
    //     uint128 _nonce
    // ) public view returns (address[] memory, uint64[] memory) {
    //     address[] memory addresses = new address[](nonce - _nonce);
    //     uint64[] memory amounts = new uint64[](nonce - _nonce);
    //     for (uint128 i = _nonce; i < nonce; i++) {
    //         // Assuming there's a way to iterate over the mapping
    //         // This is a placeholder for the actual iteration logic
    //         // addresses[i - _nonce] = msg.sender;
    //         // amounts[i - _nonce] = amount; // Assuming amount is accessible
    //     }
    //     return (addresses, amounts);
    // }

    // function pull_funds(
    //     address[] memory assets,
    //     uint64[] memory amounts,
    //     bytes memory signature,
    //     address to
    // ) public {
    //     require(msg.sender == admin, "Only the admin can pull funds");
    //     // Assuming there's a function to validate the admin signature
    //     // This is a placeholder for the actual validation logic
    //     // require(validateSignature(signature), "Invalid signature");
    //     // Increment nonce for replay protection
    //     nonce++;
    //     // Transfer assets to the specified address
    //     for (uint256 i = 0; i < assets.length; i++) {
    //         // Assuming there's a vault contract that handles the transfer
    //         // This is a placeholder for the actual vault interaction
    //         // vault.transfer(assets[i], amounts[i], to);
    //     }
    // }
}
