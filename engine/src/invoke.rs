use common::invoke::{Invocation, InvocationContext, InvocationResponse};

/// Handles a plaintext invocation.
fn handle_text_invocation(ctx: InvocationContext, payload: String) -> InvocationResponse {
    let hello_world = format!(
        "Hello {}, this is {}'s app named {} invoked at {}.",
        payload, &ctx.owner, &ctx.app, &ctx.timestamp
    );
    InvocationResponse::TextResponse {
        ctx,
        payload: hello_world,
    }
}

pub fn handle_invocation(invocation: Invocation) -> InvocationResponse {
    if let Invocation::TextInvocation { ctx, payload } = invocation {
        handle_text_invocation(ctx, payload)
    } else {
        panic!("HTTP invocations not supported yet!")
    }
}
