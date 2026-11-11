//! Concrete effectors. Each external-action capability is optional and explicitly gated.
pub mod payload;

#[cfg(feature = "effector-http")]
pub mod http;
#[cfg(feature = "effector-http")]
pub use http::HttpEffector;

#[cfg(feature = "effector-shell")]
pub mod shell;
#[cfg(feature = "effector-shell")]
pub use shell::ShellEffector;

#[cfg(feature = "effector-fs")]
pub mod filesystem;
#[cfg(feature = "effector-fs")]
pub use filesystem::FsEffector;

#[cfg(feature = "effector-code")]
pub mod code_emit;
#[cfg(feature = "effector-code")]
pub use code_emit::CodeEffector;

#[cfg(any(feature = "effector-browser-iris", feature = "effector-browser-cdp"))]
pub mod browser;
#[cfg(feature = "effector-browser-cdp")]
pub use browser::CdpBrowserEffector;
#[cfg(feature = "effector-browser-iris")]
pub use browser::IrisBrowserEffector;
