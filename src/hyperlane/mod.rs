use alloy_primitives::hex;
use alloy_primitives::Address;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DefaultHook {
    address: Address,
    #[serde(rename = "type")]
    hook_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DefaultIsm {
    address: Address,
    relayer: Address,
    #[serde(rename = "type")]
    ism_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RequiredHook {
    address: Address,
    beneficiary: Address,
    #[serde(rename = "maxProtocolFee")]
    max_protocol_fee: String,
    owner: Address,
    #[serde(rename = "protocolFee")]
    protocol_fee: String,
    #[serde(rename = "type")]
    hook_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CoreConfig {
    #[serde(rename = "defaultHook")]
    default_hook: DefaultHook,
    #[serde(rename = "defaultIsm")]
    default_ism: DefaultIsm,
    owner: Address,
    #[serde(rename = "requiredHook")]
    required_hook: RequiredHook,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct InterchainSecurityModule {
    relayer: Address,
    #[serde(rename = "type")]
    ism_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TokenType {
    Synthetic,
    FastSynthetic,
    SyntheticUri,
    Collateral,
    CollateralVault,
    XErc20,
    XErc20Lockbox,
    CollateralFiat,
    FastCollateral,
    CollateralUri,
    Native,
    NativeScaled,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChainConfig {
    #[serde(rename = "interchainSecurityModule")]
    interchain_security_module: InterchainSecurityModule,
    #[serde(rename = "isNft")]
    is_nft: bool,
    mailbox: Address,
    #[serde(rename = "interchainGasPaymaster")]
    interchain_gas_paymaster: Address,
    owner: Address,
    #[serde(rename = "type")]
    token_type: TokenType,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<Address>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WarpRouteConfig {
    #[serde(flatten)]
    chains: HashMap<String, ChainConfig>,
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("JSON deserialization error: {0}")]
    JsonDeserializationError(#[from] serde_json::Error),
    #[error("YAML deserialization error: {0}")]
    YamlDeserializationError(#[from] serde_yaml::Error),
    #[error("Invalid UTF-8")]
    InvalidUtf8,
}

impl WarpRouteConfig {
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json).map_err(ConfigError::from)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(yaml).map_err(ConfigError::from)
    }

    pub fn update_chain_config(&mut self, chain_name: &str, new_config: ChainConfig) {
        self.chains.insert(chain_name.to_string(), new_config);
    }
}

impl CoreConfig {
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json).map_err(ConfigError::from)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, ConfigError> {
        serde_yaml::from_str(yaml).map_err(ConfigError::from)
    }

    pub fn update_owner(&mut self, new_owner: Address) -> Result<(), ConfigError> {
        self.owner = new_owner;
        Ok(())
    }
}

impl TryFrom<&[u8]> for CoreConfig {
    type Error = ConfigError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let s = std::str::from_utf8(bytes).map_err(|_| ConfigError::InvalidUtf8)?;
        Self::from_yaml(s)
    }
}

impl TryFrom<&[u8]> for WarpRouteConfig {
    type Error = ConfigError;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        let s = std::str::from_utf8(bytes).map_err(|_| ConfigError::InvalidUtf8)?;
        Self::from_yaml(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::runner::run_and_focus_multiple;
    use gadget_sdk::executor::process::manager::GadgetProcessManager;

    const VALID_ADDRESS: Address = Address::new([
        0x74, 0x2d, 0x35, 0xCc, 0x66, 0x34, 0xC0, 0x53, 0x29, 0x25, 0xa3, 0xb8, 0x44, 0xBc, 0x45,
        0x4e, 0x44, 0x38, 0xf4, 0x4e,
    ]);

    fn create_sample_warp_route_config() -> WarpRouteConfig {
        WarpRouteConfig {
            chains: {
                let mut map = HashMap::new();
                map.insert(
                    "chain1".to_string(),
                    ChainConfig {
                        interchain_security_module: InterchainSecurityModule {
                            relayer: VALID_ADDRESS,
                            ism_type: "trustedRelayerIsm".to_string(),
                        },
                        is_nft: false,
                        mailbox: VALID_ADDRESS,
                        interchain_gas_paymaster: VALID_ADDRESS,
                        owner: VALID_ADDRESS,
                        token_type: TokenType::Synthetic,
                        token: Some(VALID_ADDRESS),
                    },
                );
                map
            },
        }
    }

    fn create_sample_core_config() -> CoreConfig {
        CoreConfig {
            default_hook: DefaultHook {
                address: VALID_ADDRESS,
                hook_type: "merkleTreeHook".to_string(),
            },
            default_ism: DefaultIsm {
                address: VALID_ADDRESS,
                relayer: VALID_ADDRESS,
                ism_type: "trustedRelayerIsm".to_string(),
            },
            owner: VALID_ADDRESS,
            required_hook: RequiredHook {
                address: VALID_ADDRESS,
                beneficiary: VALID_ADDRESS,
                max_protocol_fee: "100000000000000000".to_string(),
                owner: VALID_ADDRESS,
                protocol_fee: "0".to_string(),
                hook_type: "protocolFee".to_string(),
            },
        }
    }

    #[test]
    fn test_warp_route_config_serialization() {
        let config = create_sample_warp_route_config();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: WarpRouteConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_core_config_serialization() {
        let config = create_sample_core_config();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: CoreConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_warp_route_config_update() {
        let mut config = create_sample_warp_route_config();
        let new_chain_config = ChainConfig {
            interchain_security_module: InterchainSecurityModule {
                relayer: VALID_ADDRESS,
                ism_type: "newIsm".to_string(),
            },
            is_nft: true,
            mailbox: VALID_ADDRESS,
            interchain_gas_paymaster: VALID_ADDRESS,
            owner: VALID_ADDRESS,
            token_type: TokenType::Collateral,
            token: None,
        };
        config.update_chain_config("chain2", new_chain_config.clone());
        assert_eq!(config.chains.get("chain2"), Some(&new_chain_config));
    }

    #[test]
    fn test_core_config_update_owner() {
        let mut config = create_sample_core_config();
        let new_owner = VALID_ADDRESS;
        assert!(config.update_owner(new_owner).is_ok());
        assert_eq!(config.owner, new_owner);
    }

    #[test]
    fn test_warp_route_config_from_json() {
        let json = r#"
        {
            "chain1": {
                "interchainSecurityModule": {
                    "relayer": "0x742d35cc6634c0532925a3b844bc454e4438f44e",
                    "type": "trustedRelayerIsm"
                },
                "isNft": false,
                "mailbox": "0x742d35cc6634c0532925a3b844bc454e4438f44e",
                "interchainGasPaymaster": "0x742d35cc6634c0532925a3b844bc454e4438f44e",
                "owner": "0x742d35cc6634c0532925a3b844bc454e4438f44e",
                "type": "synthetic",
                "token": "0x742d35cc6634c0532925a3b844bc454e4438f44e"
            }
        }"#;
        let config = WarpRouteConfig::from_json(json).unwrap();
        assert_eq!(config.chains.len(), 1);
        assert!(config.chains.contains_key("chain1"));
    }

    #[test]
    fn test_core_config_from_yaml() {
        let yaml = r#"
        defaultHook:
          address: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          type: "merkleTreeHook"
        defaultIsm:
          address: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          relayer: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          type: "trustedRelayerIsm"
        owner: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
        requiredHook:
          address: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          beneficiary: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          maxProtocolFee: "100000000000000000"
          owner: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          protocolFee: "0"
          type: "protocolFee"
        "#;
        let config = CoreConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.owner, VALID_ADDRESS);
        assert_eq!(config.default_hook.hook_type, "merkleTreeHook");
    }

    #[test]
    fn test_warp_route_config_try_from() {
        let yaml = r#"
        chain1:
          interchainSecurityModule:
            relayer: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
            type: "trustedRelayerIsm"
          isNft: false
          mailbox: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          interchainGasPaymaster: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          owner: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          type: "synthetic"
          token: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
        "#;
        let config = WarpRouteConfig::try_from(yaml.as_bytes()).unwrap();
        assert_eq!(config.chains.len(), 1);
        assert!(config.chains.contains_key("chain1"));
    }

    #[test]
    fn test_core_config_try_from() {
        let yaml = r#"
        defaultHook:
          address: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          type: "merkleTreeHook"
        defaultIsm:
          address: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          relayer: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          type: "trustedRelayerIsm"
        owner: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
        requiredHook:
          address: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          beneficiary: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          maxProtocolFee: "100000000000000000"
          owner: "0x742d35cc6634c0532925a3b844bc454e4438f44e"
          protocolFee: "0"
          type: "protocolFee"
        "#;
        let config = CoreConfig::try_from(yaml.as_bytes()).unwrap();
        assert_eq!(config.owner, VALID_ADDRESS);
        assert_eq!(config.default_hook.hook_type, "merkleTreeHook");
    }

    #[test]
    fn test_invalid_utf8() {
        let invalid_utf8 = vec![0, 159, 146, 150]; // Invalid UTF-8 sequence
        assert!(matches!(
            WarpRouteConfig::try_from(invalid_utf8.as_slice()),
            Err(ConfigError::InvalidUtf8)
        ));
        assert!(matches!(
            CoreConfig::try_from(invalid_utf8.as_slice()),
            Err(ConfigError::InvalidUtf8)
        ));
    }
}
