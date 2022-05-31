use common::{
    errors::InvocationError,
    invoke::{Invocation, InvocationContext, InvocationResponse},
};
use shared::http::HttpReq;

use crate::core::Core;

/// Handles a plaintext invocation.
fn handle_text_invocation(
    ctx: InvocationContext,
    payload: String,
    core: &mut Core,
) -> Result<InvocationResponse, InvocationError> {
    let response = core.invoke_text(&payload)?;
    Ok(InvocationResponse::TextResponse {
        ctx,
        payload: response,
    })
}
/// Handles an HTTP invocation.
fn handle_http_invocation(
    ctx: InvocationContext,
    payload: HttpReq,
    core: &mut Core,
) -> Result<InvocationResponse, InvocationError> {
    let response = core.invoke_http(&payload)?;
    Ok(InvocationResponse::HttpResponse {
        ctx,
        payload: response,
    })
}

pub fn handle_invocation(
    invocation: Invocation,
    core: &mut Core,
) -> Result<InvocationResponse, InvocationError> {
    match invocation {
        Invocation::TextInvocation { ctx, payload } => handle_text_invocation(ctx, payload, core),
        Invocation::HttpInvocation { ctx, payload } => handle_http_invocation(ctx, payload, core),
    }
}
