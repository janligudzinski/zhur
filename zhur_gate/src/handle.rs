use conversions::realize_response;
use hyper::{Body, Request, Response};
use std::convert::Infallible;
use zhur_common::{log::*, msg::chan::ChannelClient};
use zhur_invk::*;
mod conversions;
/// The info we need to produce a Zhur invocation from an HTTP request.
pub struct FullRequest {
    /// The request itself.
    pub req: Request<Body>,
    /// The IP address the request is coming from, represented as a string.
    pub ip: String,
}

/// Transforms an HTTP request into an HTTP response.
pub async fn handle_req(
    req: FullRequest,
    mut client: ChannelClient<Invocation, Result<HttpRes, InvocationError>>,
) -> Result<Response<Body>, Infallible> {
    let invocation = match req.into_invoc().await {
        Ok(i) => {
            let text = format!(
                "Got an OK invocation for {}:{} of length {}",
                &i.owner,
                &i.app_name,
                &i.payload.len()
            );
            info!("{}", &text);
            i
        }
        Err(e) => {
            let text = format!("Got an invocation error: {}", e);
            warn!("{}", &text);
            return Ok(Response::new(text.into())); // TODO: Error pages
        }
    };
    let reply = client.request(invocation);
    match reply {
        Ok(res) => {
            info!("Got a well-formed HttpRes as an invocation result!");
            return Ok(
                realize_response(res)
            );
        }
        Err(e) => {
            let text = format!("Got an invocation error: {}", e);
            warn!("{}", &text);
            return Ok(Response::new(text.into())); // TODO: Error pages
                                                   // TODO: Not brazen copypaste
        }
    }
}
