use serde::Deserialize;
use serde::Serialize;

#[derive(clap::ArgEnum, Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) enum HostMode {
    /// Trivial Wasm Host Environment
    #[default]
    Default,

    /// Wasm Host Environment with more Zk plugins
    Standard,
}

#[derive(clap::ArgEnum, Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) enum Scheme {
    Gwc,

    #[default]
    Shplonk,
}
