use gadget_sdk::config::GadgetConfiguration;
use gadget_sdk::executor::process::manager::GadgetProcessManager;
use gadget_sdk::job;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::OnceLock;
use std::collections::HashMap;

pub mod runner;
use runner::run_and_focus_multiple;

static HYP_KEY: OnceLock<String> = OnceLock::new();

fn hyperlane_key() -> &'static str {
    HYP_KEY.get_or_init(|| {
        std::env::var("HYP_KEY").expect("HYP_KEY environment variable not set")
    })
}

/// These jobs represent the Hyperlane blueprint functions. The goal is to instance
/// a set of operators to manage different Hyperlane services, including
/// validating and relaying.
///
/// Hyperlane core configuration example:
/// ```yaml
/// defaultHook:
///   address: "0x87b863A6A0d841F2949f03016c306f4b9d346feD"
///   type: merkleTreeHook
/// defaultIsm:
///   address: "0x3Bdf0feB98E02edC12650D3938DBF392C062884A"
///   relayer: "0x14526fE72F9560716f935D0d8E51B5E3568A8836"
///   type: trustedRelayerIsm
/// owner: "0x009928463436d9CFf320d4E9E0D36376c71C1C1E"
/// requiredHook:
///   address: "0x5a3Fe91ea1b337baF23aca05606A6c1D05D270E8"
///   beneficiary: "0x009928463436d9CFf320d4E9E0D36376c71C1C1E"
///   maxProtocolFee: "100000000000000000"
///   owner: "0x009928463436d9CFf320d4E9E0D36376c71C1C1E"
///   protocolFee: "0"
///   type: protocolFee
/// ```

/// Hyperlane allows you to create a route to transfer tokens between chains. This service
/// creates new routes, which all instance operators then validate and relay.
///
/// To create a new route:
/// 1. Verify Hyperlane deployment on both chains
/// 2. If not deployed, handle deployment or prompt for deployment confirmation
/// 3. Create the route using the provided configuration
/// 4. Validate the new route
/// 5. Set up relayers for the new route
///
/// Warp configuration example:
/// ```yaml
/// holesky:
///   interchainSecurityModule:
///     relayer: "0x14526fE72F9560716f935D0d8E51B5E3568A8836"
///     type: trustedRelayerIsm
///   isNft: false
///   mailbox: "0x57529d3663bb44e8ab3335743dd42d2e1E3b46BA"
///   interchainGasPaymaster: "0x5CBf4e70448Ed46c2616b04e9ebc72D29FF0cfA9"
///   owner: "0x009928463436d9CFf320d4E9E0D36376c71C1C1E"
///   token: "0x94373a4919B3240D86eA41593D5eBa789FEF3848"
///   type: collateral
/// tangletestnet:
///   interchainSecurityModule:
///     relayer: "0x14526fE72F9560716f935D0d8E51B5E3568A8836"
///     type: trustedRelayerIsm
///   isNft: false
///   mailbox: "0x0FDc2400B5a50637880dbEfB25d631c957620De8"
///   interchainGasPaymaster: "0x0000000000000000000000000000000000000000"
///   owner: "0x009928463436d9CFf320d4E9E0D36376c71C1C1E"
///   type: synthetic
/// ```
///
/// The service should handle different token types (collateral, synthetic, or NFT) and
/// ensure proper setup for each case. It should also manage the differences in configuration
/// between chains, such as the presence or absence of an interchainGasPaymaster.
#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultHook {
    address: String,
    #[serde(rename = "type")]
    hook_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultIsm {
    address: String,
    relayer: String,
    #[serde(rename = "type")]
    ism_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequiredHook {
    address: String,
    beneficiary: String,
    #[serde(rename = "maxProtocolFee")]
    max_protocol_fee: String,
    owner: String,
    #[serde(rename = "protocolFee")]
    protocol_fee: String,
    #[serde(rename = "type")]
    hook_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreConfig {
    #[serde(rename = "defaultHook")]
    default_hook: DefaultHook,
    #[serde(rename = "defaultIsm")]
    default_ism: DefaultIsm,
    owner: String,
    #[serde(rename = "requiredHook")]
    required_hook: RequiredHook,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct InterchainSecurityModule {
    relayer: String,
    #[serde(rename = "type")]
    ism_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChainConfig {
    #[serde(rename = "interchainSecurityModule")]
    interchain_security_module: InterchainSecurityModule,
    #[serde(rename = "isNft")]
    is_nft: bool,
    mailbox: String,
    #[serde(rename = "interchainGasPaymaster")]
    interchain_gas_paymaster: String,
    owner: String,
    #[serde(rename = "type")]
    chain_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WarpRouteConfig {
    #[serde(flatten)]
    chains: HashMap<String, ChainConfig>,
}

impl WarpRouteConfig {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl CoreConfig {
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(yaml)
    }
}

impl TryFrom<&[u8]> for WarpRouteConfig {
    type Error = serde_json::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(bytes)
    }
}

impl TryFrom<&[u8]> for CoreConfig {
    type Error = serde_json::Error;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        serde_json::from_slice(bytes)
    }
}


#[job(
    id = 0,
    params(config, advanced, use_existing_core_config),
    result(_),
    verifier(evm = "HyperlaneBlueprint")
)]
pub async fn operate_a_warp_route(
    config: Vec<u8>,
    advanced: bool,
    use_existing_core_config: Vec<u8>,
    env: GadgetConfiguration<parking_lot::RawRwLock>,
) -> Result<u64, Infallible> {
    // 1. Deploy or use an existing set of Hyperlane contracts
    //     `hyperlane registry init`
    //     `hyperlane core init --advanced [config]` for non-trusted relayer setup
    //     `hyperlane core init` just gives you a trusted relayer setup (relayer address is deployer)
    //     `hyperlane core deploy`
    let mut manager = GadgetProcessManager::new();
    if use_existing_core_config.is_empty() {
        let commands = vec![
            ("run registry init", "hyperlane registry init"),
            ("run core init advanced", "hyperlane core init --advanced [config]"),
            ("run core deploy", "hyperlane core deploy"),
        ];
        let outputs = run_and_focus_multiple(&mut manager, commands).await.unwrap();
    } else {
        // Deserialize the existing core config
        let core_config = CoreConfig::try_from(&use_existing_core_config[..])
            .unwrap_or_else(|e| {
                eprintln!("Failed to deserialize existing core config: {}", e);
                std::process::exit(1);
            });

        // Log the deserialized core config for debugging
        println!("Deserialized existing core config: {:?}", core_config);

        // Use the existing core config in subsequent operations
        let commands = vec![
            ("run registry init", "hyperlane registry init"),
            ("run core init --advanced", "hyperlane core init --advanced"),
            ("run core deploy", "hyperlane core deploy"),
        ];
        let outputs = run_and_focus_multiple(&mut manager, commands).await.unwrap();
    }


    // 2. `hyperlane warp init` - Initialize the Hyperlane warp route
    // Deserialize the config into the WarpRouteConfig struct
    let warp_route_config = WarpRouteConfig::try_from(&config[..])
        .unwrap_or_else(|e| {
            eprintln!("Failed to deserialize config: {}", e);
            std::process::exit(1);
        });

    // Log the deserialized config for debugging
    println!("Deserialized WarpRouteConfig: {:?}", warp_route_config);

    // 3. `hyperlane warp deploy` - Deploy the Hyperlane warp route
    let should_i_deploy = true; // Decide if this operator should deploy the warp route
    if should_i_deploy {
        let commands = vec![
            ("run warp deploy", "hyperlane warp deploy"),
         ];
         let outputs = run_and_focus_multiple(&mut manager, commands).await.unwrap();
    }

    // 4. Update the core config of Hyperlane contracts on those chains
    // i.e. on Holesky we do
    //      `hyperlane core read --chain holesky`
    //      `hyperlane core apply --chain holesky`
    // i.e. on Tangle we do:
    //     `hyperlane core read --chain tangle`
    //     `hyperlane core apply --chain tangle`
    // 
    // Note: Core apply can only be run by the person who deployed hyperlane core contracts
    let mut outputs = HashMap::new();

    // Read Holesky core config
    let holesky_read_command = ("run core read --chain holesky", "hyperlane core read --chain holesky");
    outputs.insert(holesky_read_command.0.to_string(), run_and_focus_multiple(&mut manager, vec![holesky_read_command]).await.unwrap().remove(holesky_read_command.0).unwrap());

    // Apply Holesky core config
    let holesky_apply_command = ("run core apply --chain holesky", format!("hyperlane core apply --chain holesky --input '{}'", outputs["run core read --chain holesky"]));
    run_and_focus_multiple(&mut manager, vec![(holesky_apply_command.0, &holesky_apply_command.1)]).await.unwrap();

    // Read Tangle core config
    let tangle_read_command = ("run core read --chain tangletestnet", "hyperlane core read --chain tangletestnet");
    outputs.insert(tangle_read_command.0.to_string(), run_and_focus_multiple(&mut manager, vec![tangle_read_command]).await.unwrap().remove(tangle_read_command.0).unwrap());

    // Apply Tangle core config
    let tangle_apply_command = ("run core apply --chain tangletestnet", format!("hyperlane core apply --chain tangletestnet --input '{}'", outputs["run core read --chain tangletestnet"]));
    run_and_focus_multiple(&mut manager, vec![(tangle_apply_command.0, &tangle_apply_command.1)]).await.unwrap();

    Ok(0)
}
