use color_eyre::Result;
use gadget_sdk as sdk;
pub use hyperlane_blueprint_template as blueprint;
use sdk::ctx::TangleClientContext;
use sdk::info;
use sdk::job_runner::MultiJobRunner;
use sdk::tangle_subxt::subxt::tx::Signer;
use std::sync::Arc;

#[sdk::main(env)]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let ctx = Arc::new(blueprint::HyperlaneContext { env });

    let client = ctx.tangle_client().await?;
    let signer = ctx.env.first_sr25519_signer()?;

    let start_warp_route = blueprint::OperateAWarpRouteEventHandler {
        ctx: Arc::clone(&ctx),
        service_id: ctx.env.service_id.unwrap(),
        signer: signer.clone(),
        client,
    };

    info!("Starting the event watcher for {} ...", signer.account_id());

    MultiJobRunner::new(ctx.env.clone())
        .job(start_warp_route)
        .run()
        .await?;

    Ok(())
}
