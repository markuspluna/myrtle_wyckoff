// EIP-712 implementation
// might need a domain separator for different orderbooks but unsure

use alloy::{primitives::Address, sol_types::eip712_domain};

pub const MAINNET_DOMAIN: alloy::dyn_abi::Eip712Domain = eip712_domain! {
    name: "MyrtleWyckoff",
    version: "1",
    verifying_contract: Address::ZERO, //TODO replace with deposit contract address
};

pub const TOLIMAN_DOMAIN: alloy::dyn_abi::Eip712Domain = eip712_domain! {
    name: "MyrtleWyckoff",
    version: "1",
    verifying_contract: Address::ZERO, //TODO replace with checkpoint contract address
};

pub const DSTACK_DOMAIN: alloy::dyn_abi::Eip712Domain = eip712_domain! {
    name: "MyrtleWyckoff",
    version: "1",
    verifying_contract: Address::ZERO,
};
