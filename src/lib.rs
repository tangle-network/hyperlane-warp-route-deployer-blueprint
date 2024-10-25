use gadget_sdk as sdk;
use sdk::config::StdGadgetConfiguration;
use sdk::ctx::{ServicesContext, TangleClientContext};
use sdk::event_listener::tangle::jobs::{services_post_processor, services_pre_processor};
use sdk::event_listener::tangle::TangleEventListener;
use sdk::executor::process::manager::GadgetProcessManager;
use sdk::tangle_subxt::tangle_testnet_runtime::api::services::events::JobCalled;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, LazyLock};

pub mod hyperlane;
use crate::hyperlane::{CoreConfig, WarpRouteConfig};

pub mod runner;
use runner::run_and_focus_multiple;

static HYPERLANE_KEY: LazyLock<String> =
    LazyLock::new(|| std::env::var("HYP_KEY").expect("HYP_KEY environment variable not set"));

#[derive(TangleClientContext, ServicesContext)]
pub struct HyperlaneContext {
    #[config]
    pub env: StdGadgetConfiguration,
}

#[sdk::job(
    id = 0,
    params(config, advanced, existing_core_config),
    result(_),
    event_listener(
        listener = TangleEventListener<JobCalled, Arc<HyperlaneContext>>,
        pre_processor = services_pre_processor,
        post_processor = services_post_processor,
    ),
)]
pub async fn operate_a_warp_route(
    ctx: Arc<HyperlaneContext>,
    config: Vec<u8>,
    advanced: bool,
    existing_core_config: Option<Vec<u8>>,
) -> Result<u64, Infallible> {
    // 1. Deploy or use an existing set of Hyperlane contracts
    //     `hyperlane registry init`
    //     `hyperlane core init --advanced [config]` for non-trusted relayer setup
    //     `hyperlane core init` just gives you a trusted relayer setup (relayer address is deployer)
    //     `hyperlane core deploy`
    let mut manager = GadgetProcessManager::new();
    match existing_core_config {
        Some(existing_core_config) => {
            // Deserialize the existing core config
            let core_config = CoreConfig::try_from(&existing_core_config[..]).unwrap_or_else(|e| {
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
            let outputs = run_and_focus_multiple(&mut manager, commands)
                .await
                .unwrap();
        }
        None => {
            let commands = vec![
                ("run registry init", "hyperlane registry init"),
                (
                    "run core init advanced",
                    "hyperlane core init --advanced [config]",
                ),
                ("run core deploy", "hyperlane core deploy"),
            ];
            let outputs = run_and_focus_multiple(&mut manager, commands)
                .await
                .unwrap();
        }
    }

    // 2. `hyperlane warp init` - Initialize the Hyperlane warp route
    // Deserialize the config into the WarpRouteConfig struct
    let warp_route_config = WarpRouteConfig::try_from(&config[..]).unwrap_or_else(|e| {
        eprintln!("Failed to deserialize config: {}", e);
        std::process::exit(1);
    });

    // Log the deserialized config for debugging
    println!("Deserialized WarpRouteConfig: {:?}", warp_route_config);

    // 3. `hyperlane warp deploy` - Deploy the Hyperlane warp route
    let should_i_deploy = true; // Decide if this operator should deploy the warp route
    if should_i_deploy {
        let commands = vec![("run warp deploy", "hyperlane warp deploy")];
        let outputs = run_and_focus_multiple(&mut manager, commands)
            .await
            .unwrap();
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
    let holesky_read_command = (
        "run core read --chain holesky",
        "hyperlane core read --chain holesky",
    );
    outputs.insert(
        holesky_read_command.0.to_string(),
        run_and_focus_multiple(&mut manager, vec![holesky_read_command])
            .await
            .unwrap()
            .remove(holesky_read_command.0)
            .unwrap(),
    );

    // Apply Holesky core config
    let holesky_apply_command = (
        "run core apply --chain holesky",
        format!(
            "hyperlane core apply --chain holesky --input '{}'",
            outputs["run core read --chain holesky"]
        ),
    );
    run_and_focus_multiple(
        &mut manager,
        vec![(holesky_apply_command.0, &holesky_apply_command.1)],
    )
    .await
    .unwrap();

    // Read Tangle core config
    let tangle_read_command = (
        "run core read --chain tangletestnet",
        "hyperlane core read --chain tangletestnet",
    );
    outputs.insert(
        tangle_read_command.0.to_string(),
        run_and_focus_multiple(&mut manager, vec![tangle_read_command])
            .await
            .unwrap()
            .remove(tangle_read_command.0)
            .unwrap(),
    );

    // Apply Tangle core config
    let tangle_apply_command = (
        "run core apply --chain tangletestnet",
        format!(
            "hyperlane core apply --chain tangletestnet --input '{}'",
            outputs["run core read --chain tangletestnet"]
        ),
    );
    run_and_focus_multiple(
        &mut manager,
        vec![(tangle_apply_command.0, &tangle_apply_command.1)],
    )
    .await
    .unwrap();

    Ok(0)
}
