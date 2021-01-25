/// Herein lives the `ApstServer`, which responds to requests for apps and issues updates when they are changed.
mod serve;
pub use serve::ApstServer;
pub use zhur_common::msg::core_apst::DEFAULT_APST_ENDPOINT as DEFAULT_ENDPOINT;