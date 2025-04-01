use std::path::PathBuf;

use crate::args::Scheme;

use clap::Args;

use crate::args::HostMode;

#[derive(Debug)]
pub(crate) struct SetupArg {
    pub(crate) k: u32,
    pub(crate) host_mode: HostMode,
    pub(crate) phantom_functions: Vec<String>,
    pub(crate) wasm_image: Option<PathBuf>,
    pub(crate) scheme: Scheme,
}

#[derive(Debug, Args)]
pub(crate) struct RunningArg {
    /// Path to the directory to write the output.
    #[clap(short = 'o', long = "output")]
    pub(crate) output_dir: PathBuf,

    /// Public inputs with format 'value:type' where type=i64|bytes|bytes-packed|file.
    #[clap(long = "public")]
    pub(crate) public_inputs: Vec<String>,

    /// Private inputs with format 'value:type' where type=i64|bytes|bytes-packed|file.
    #[clap(long = "private")]
    pub(crate) private_inputs: Vec<String>,

    /// Context inputs with format 'value:type' where type=i64|bytes|bytes-packed|file.
    #[clap(long = "context-in")]
    pub(crate) context_inputs: Vec<String>,

    /// Filename to the file to write the context output.
    #[clap(long = "context-out")]
    pub(crate) context_output: Option<String>,
}

#[derive(Debug)]
pub(crate) struct DryRunArg {
    pub(crate) wasm_image: PathBuf,
    pub(crate) running_arg: RunningArg,
    pub(crate) instruction_limit: Option<usize>,
}

/// Execute the Wasm image and generate a proof.
#[derive(Debug)]
pub(crate) struct ProveArg {
    pub(crate) wasm_image: PathBuf,
    pub(crate) output_dir: PathBuf,
    pub(crate) running_arg: RunningArg,
    pub(crate) mock_test: bool,
    pub(crate) file_backend: bool,
    // skip first n slice(s) proving.
    pub(crate) skip: usize,
    // add trivial circuits to padding
    pub(crate) padding: Option<usize>,
}

/// Verify the proof.
#[derive(Debug, Args)]
pub(crate) struct VerifyArg {
    /// Path to the directory to proof.
    #[clap(short = 'o', long = "output")]
    pub(crate) output_dir: PathBuf,
}

#[derive(Debug)]
pub(crate) enum Subcommands {
    Setup(SetupArg),
    DryRun(DryRunArg),
    Prove(ProveArg),
    Verify(VerifyArg),
}
