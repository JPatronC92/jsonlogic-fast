use alloy::sol;
use alloy::providers::{ProviderBuilder, RootProvider};
use alloy::network::EthereumWallet;
use alloy::signers::local::PrivateKeySigner;
use alloy::transports::http::{Client, Http};
use std::env;
use std::sync::Arc;

sol! {
    #[allow(missing_docs)]
    #[sol(rpc)]
    contract B2AStaking {
        mapping(address => uint256) public balances;
        event Deposited(address indexed user, uint256 amount);
        event Slashed(address indexed user, uint256 amount);
        function deposit() public payable;
        function slash(address user, uint256 amount) public;
    }
}

pub struct BlockchainConfig {
    pub rpc_url: String,
    pub private_key: String,
    pub contract_address: alloy::primitives::Address,
}

impl BlockchainConfig {
    pub fn from_env() -> Self {
        let rpc_url = env::var("RPC_URL").unwrap_or_else(|_| "https://sepolia.base.org".to_string());
        let private_key = env::var("PRIVATE_KEY").unwrap_or_else(|_| "0000000000000000000000000000000000000000000000000000000000000001".to_string());
        let contract_address_str = env::var("CONTRACT_ADDRESS").unwrap_or_else(|_| "0x0000000000000000000000000000000000000000".to_string());

        let contract_address = contract_address_str.parse().expect("Invalid contract address");

        Self {
            rpc_url,
            private_key,
            contract_address,
        }
    }

    pub fn read_provider(&self) -> Arc<impl alloy::providers::Provider> {
        Arc::new(ProviderBuilder::new().connect_http(self.rpc_url.parse().unwrap()))
    }

    pub fn write_provider(&self) -> Arc<impl alloy::providers::Provider> {
        let signer: PrivateKeySigner = self.private_key.parse().expect("Invalid private key");
        let wallet = EthereumWallet::from(signer);
        Arc::new(
            ProviderBuilder::new()
                .wallet(wallet)
                .connect_http(self.rpc_url.parse().unwrap())
        )
    }
}
