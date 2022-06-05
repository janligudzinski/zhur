use clap::Parser;
use common::{
    invoke::{Invocation, InvocationContext, InvocationResponse},
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
        let ctx = InvocationContext::new(flags.owner.clone(), flags.app_name.clone());
        let invocation = Invocation::TextInvocation { ctx, payload: name };
        let response = client.request::<_, InvocationResponse>(&invocation).await?;
        match response {
            InvocationResponse::TextResponse { ctx: _, payload } => {
                info!("Got text response from engine:\n{}", payload);
            }
            InvocationResponse::HttpResponse { ctx: _, payload: _ } => {
                info!("Got HTTP response from engine.");
            }
        }
    }
    Ok(())
}
