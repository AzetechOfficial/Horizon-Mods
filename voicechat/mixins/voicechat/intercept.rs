// Voice chat messages never reach the game's decoder: they are queued for the
// client's tick, and an empty body (which the reader skips) goes on instead.
fn voicechat_intercept(
    frame: &[u8],
    threshold: Option<usize>,
) -> core::ops::ControlFlow<Result<Vec<u8>>> {
    if crate::frame::voicechat_payloads::intercept(frame, threshold) {
        core::ops::ControlFlow::Break(Ok(Vec::new()))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
