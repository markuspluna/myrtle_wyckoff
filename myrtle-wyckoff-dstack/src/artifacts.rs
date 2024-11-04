use alloy::sol;

sol! (
    #[sol(rpc)]
    interface IDepositRegistry {
        #[derive(serde::Deserialize)]
        struct Order {
            uint256 ethAmount;
            uint256 usdcAmount;
            bool isBid;
            uint256 nonce;
        }

        function GPv2Settlement() external view returns (address);
        function HookTrampoline() external view returns (address);
        function USDC() external view returns (address);
        function WETH() external view returns (address);
        function admin() external view returns (address);

        function deposit(
            address user,
            uint256 ethAmount,
            uint256 usdcAmount
        ) external;

        function deposit_registry(
            address user,
            uint256 nonce,
            uint256 index
        ) external view returns (uint256);

        function get_deposits(
            uint32 nonce,
            address user
        ) external view returns (uint256[2][] memory);

        function isValidSignature(
            bytes32 hash,
            bytes memory signature
        ) external view returns (bytes4);

        function pull_settlement_funds(
            Order calldata order,
            bytes calldata signature
        ) external;

        function set_admin(address newAdmin) external;
        function settlement_nonce() external view returns (uint256);
    }
);

sol! (
    #[sol(rpc)]
    interface ICheckpointer {
    struct Checkpoint {
        uint256 nonce;
        uint8[] inventory_state;
        string[] settlement_orders;
    }

    event SettlementOrders(string[] settlement_orders);

    function admin() external view returns (address);
    function inventory_checkpoint_nonce() external view returns (uint256);
    function inventory_checkpoint(uint256) external view returns (uint8);
    function set_admin(address new_admin) external;
    function checkpoint(bytes calldata signature, Checkpoint calldata _checkpoint) external;
}
);
