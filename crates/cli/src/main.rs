#![deny(warnings)]
#![allow(clippy::too_many_arguments, clippy::while_let_on_iterator)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::Result;
use app_builder::app;
use command::Subcommands;
use delphinus_zkwasm::runtime::host::default_env::DefaultHostEnvBuilder;
use delphinus_zkwasm::runtime::host::default_env::ExecutionArg;

use args::HostMode;
use config::Config;
use delphinus_zkwasm::runtime::host::HostEnvBuilder;
use file_backend::FileBackendBuilder;
use specs::args::parse_args;
use specs::slice_backend::InMemoryBackendBuilder;

mod app_builder;
mod args;
mod command;
mod config;
mod file_backend;
mod names;

pub mod utils;

const TRIVIAL_WASM: &str = r#"
(module
    (func (export "zkmain"))
)
"#;

#[derive(Debug)]
struct ZkWasmCli {
    name: String,
    params_dir: PathBuf,
    subcommand: Subcommands,
}

/// Simple program to greet a person
fn main() -> Result<()> {
    {
        env_logger::init();
    }

    let app = app();

    let cli: ZkWasmCli = app.get_matches().into();

    match cli.subcommand {
        Subcommands::Setup(arg) => {
            let env_builder: Box<dyn HostEnvBuilder> = match arg.host_mode {
                HostMode::Default => Box::new(DefaultHostEnvBuilder::new(arg.k)),
                HostMode::Standard => unimplemented!(),
            };

            arg.setup(&*env_builder, &cli.name, &cli.params_dir)?;
        }
        // Subcommands::DryRun(_) => unimplemented!(),
        Subcommands::Prove(arg) => {
            let trace_dir = arg.output_dir.join("traces");
            fs::create_dir_all(&trace_dir)?;

            let mut config = Config::default();

            let public_inputs = parse_args(&arg.running_arg.public_inputs);
            let private_inputs = parse_args(&arg.running_arg.private_inputs);
            let context_inputs = parse_args(&arg.running_arg.context_inputs);

            // TODO: revise to args.
            config.k = 18;
            let env_builder: Box<dyn HostEnvBuilder> =
                Box::new(DefaultHostEnvBuilder::new(config.k));

            if arg.file_backend {
                let backend_builder = FileBackendBuilder::new(cli.name.clone(), trace_dir);

                config.prove(
                    backend_builder,
                    &*env_builder,
                    &arg.wasm_image,
                    &arg.output_dir,
                    ExecutionArg {
                        public_inputs,
                        private_inputs,
                        context_inputs,
                        indexed_witness: Rc::new(RefCell::new(HashMap::default())),
                        // tree_db: Some(Rc::new(RefCell::new(MongoDB::new([0; 32], None)))),
                    },
                    arg.running_arg.context_output,
                    arg.mock_test,
                    arg.skip,
                    arg.padding,
                )?;
            } else {
                let backend_builder = InMemoryBackendBuilder;

                config.prove(
                    backend_builder,
                    &*env_builder,
                    &arg.wasm_image,
                    &arg.output_dir,
                    ExecutionArg {
                        public_inputs,
                        private_inputs,
                        context_inputs,
                        indexed_witness: Rc::new(RefCell::new(HashMap::default())),
                        // tree_db: Some(Rc::new(RefCell::new(MongoDB::new([0; 32], None)))),
                    },
                    arg.running_arg.context_output,
                    arg.mock_test,
                    arg.skip,
                    arg.padding,
                )?;
            }
        } // Subcommands::Verify(_) => unimplemented!(),
    }

    Ok(())
}
