use clap::Parser;
use common::{
    invoke::{Invocation, InvocationContext, InvocationType::Json, JsonResponse},
    prelude::*,
};
use ipc::UnixClient;
use log::*;
#[derive(Parser, Debug)]
struct Flags {
    #[clap(short, long)]
    owner: String,
    #[clap(short, long)]
    app_name: String,
    #[clap(short, long)]
    payload: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init()?;
    let flags = Flags::try_parse()?;
    let stream = tokio::net::UnixStream::connect("/tmp/zhur-engine.sck").await?;
    let mut client = UnixClient::new(1024 * 8, stream);
    let names = vec!["Alice".to_string(), "Bob".to_string(), "Carol".to_string()];
    for name in names {
        let ctx = InvocationContext::new(flags.owner.clone(), flags.app_name.clone(), Json);
        let invocation = Invocation::new(ctx, &name);
        let response = client.request::<_, JsonResponse>(&invocation).await?;
        info!("Got response from engine:\n{}", response.payload);
    }
    Ok(())
}
