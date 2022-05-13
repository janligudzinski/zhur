use common::{
    errors::InvocationError,
    invoke::{Invocation, InvocationContext, InvocationResponse},
};

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

pub fn handle_invocation(
    invocation: Invocation,
    core: &mut Core,
) -> Result<InvocationResponse, InvocationError> {
    if let Invocation::TextInvocation { ctx, payload } = invocation {
        handle_text_invocation(ctx, payload, core)
    } else {
        panic!("HTTP invocations not supported yet!")
    }
}
