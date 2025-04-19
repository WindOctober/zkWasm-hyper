use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use console::style;
use delphinus_zkwasm::circuits::ZkWasmCircuit;
use delphinus_zkwasm::loader::slice::Slices;
use delphinus_zkwasm::loader::Module;
use delphinus_zkwasm::loader::ZkWasmLoader;
use delphinus_zkwasm::runtime::host::default_env::ExecutionArg;
use delphinus_zkwasm::runtime::host::HostEnvBuilder;
use delphinus_zkwasm::runtime::monitor::table_monitor::TableMonitor;
use halo2_proofs::pairing::bn256::Bn256;
use plonkish_backend::backend;
use plonkish_backend::backend::PlonkishBackend;
use plonkish_backend::backend::PlonkishCircuit;
use plonkish_backend::halo2_curves::bn256::Bn256 as PBN256;
use plonkish_backend::pcs::multilinear;
use plonkish_backend::pcs::univariate;
use plonkish_backend::transform::circuit::get_zkwasm_circuit;
use plonkish_backend::util::end_timer;
use plonkish_backend::util::start_timer;
use plonkish_backend::util::transcript::InMemoryTranscript;
use plonkish_backend::util::transcript::Keccak256Transcript;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::Deserialize;
use serde::Serialize;
use specs::slice_backend::SliceBackendBuilder;

use crate::args::HostMode;
use crate::args::Scheme;

use crate::names::name_of_etable_slice;
use crate::names::name_of_external_host_call_table_slice;
use crate::names::name_of_frame_table_slice;
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct CircuitDataMd5 {
    pub(crate) circuit_data_md5: String,
    pub(crate) verifying_key_md5: String,
}

#[cfg(not(feature = "continuation"))]
#[derive(Default, Serialize, Deserialize)]
pub(crate) struct CircuitDataConfig {
    pub(crate) finalized_circuit: CircuitDataMd5,
}

#[cfg(feature = "continuation")]
#[derive(Serialize, Deserialize)]
pub(crate) struct CircuitDataConfig {
    pub(crate) on_going_circuit: CircuitDataMd5,
    pub(crate) finalized_circuit: CircuitDataMd5,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Config {
    pub(crate) name: String,

    pub(crate) is_uniform_circuit: bool,
    pub(crate) k: u32,
    pub(crate) params: PathBuf,
    pub(crate) params_md5: String,
    pub(crate) wasm_image_md5: Option<String>,
    pub(crate) circuit_datas: CircuitDataConfig,

    pub(crate) checksum: (String, String),
    pub(crate) phantom_functions: Vec<String>,
    pub(crate) host_mode: HostMode,

    pub(crate) scheme: Scheme,
}

impl Config {
    fn image_consistent_check(&self, wasm_image: &[u8]) -> anyhow::Result<()> {
        if let Some(expected_wasm_image_md5) = &self.wasm_image_md5 {
            let wasm_image_md5 = format!("{:x}", md5::compute(wasm_image));

            if expected_wasm_image_md5 != &wasm_image_md5 {
                anyhow::bail!(
                    "Wasm image is inconsistent with the one used to build the circuit. \
                        Maybe you have changed the Wasm image after setup the circuit?",
                );
            }
        }

        Ok(())
    }
}

impl Config {
    pub(crate) fn write(&self, fd: &mut File) -> anyhow::Result<()> {
        fd.write_all(&bincode::serialize(self)?)?;

        Ok(())
    }

    // pub(crate) fn read(fd: &mut File) -> anyhow::Result<Self> {
    //     let mut buf = Vec::new();
    //     fd.read_to_end(&mut buf)?;
    //     let config = bincode::deserialize(&buf)?;

    //     Ok(config)
    // }
}

impl Config {
    fn read_wasm_image(&self, wasm_image: &Path) -> anyhow::Result<Module> {
        let mut buf = Vec::new();
        File::open(wasm_image)?.read_to_end(&mut buf)?;

        self.image_consistent_check(&buf)?;

        ZkWasmLoader::parse_module(&buf)
    }

    pub(crate) fn prove<B: SliceBackendBuilder>(
        self,
        slice_backend_builder: B,
        env_builder: &dyn HostEnvBuilder,
        wasm_image: &Path,
        output_dir: &Path,
        arg: ExecutionArg,
        context_output_filename: Option<String>,
        mock_test: bool,
        skip: usize,
        padding: Option<usize>,
    ) -> anyhow::Result<()> {
        println!("{} Load image...", style("[1/8]").bold().dim(),);
        let module = self.read_wasm_image(wasm_image)?;

        let env = env_builder.create_env(arg);

        let mut monitor = TableMonitor::new(
            self.k,
            slice_backend_builder,
            env_builder.create_flush_strategy(),
            &self.phantom_functions,
            &env,
        );

        let (result, tables) = {
            println!("{} Executing...", style("[3/8]").bold().dim(),);

            let loader = ZkWasmLoader::new(self.k, env)?;
            let runner = loader.compile(&module, &mut monitor)?;
            let result = loader.run(runner, &mut monitor)?;

            println!("total guest instructions used {:?}", result.guest_statics);
            println!("total host api used {:?}", result.host_statics);

            (result, monitor.into_tables())
        };

        {
            if let Some(context_output_filename) = context_output_filename {
                let context_output_path = output_dir.join(context_output_filename);

                println!(
                    "{} Write context output to file {:?}...",
                    style("[4/8]").bold().dim(),
                    context_output_path
                );

                result
                    .context_outputs
                    .write(&mut File::create(&context_output_path)?)?;
            } else {
                println!(
                    "{} Context output is not specified. Skip writing context output...",
                    style("[4/8]").bold().dim()
                );
            }
        }

        {
            let dir = output_dir.join("traces");

            println!(
                "{} Writing traces to {:?}...",
                style("[5/8]").bold().dim(),
                dir
            );
            tables.write(
                &dir,
                |index| name_of_frame_table_slice(&self.name, index),
                |index| name_of_etable_slice(&self.name, index),
                |index| name_of_external_host_call_table_slice(&self.name, index),
            )?;
        }

        println!("{} Build circuit(s)...", style("[6/8]").bold().dim(),);
        let instances = result
            .public_inputs_and_outputs
            .iter()
            .map(|v| (*v).into())
            .collect::<Vec<_>>();

        println!("{} Creating proof(s)...", style("[7/8]").bold().dim(),);

        // let mut proof_load_info = ProofGenerationInfo::new(&self.name, self.k as usize);
        if skip != 0 {
            println!("skip first {} slice(s)", skip);
        }

        let mut slices = Slices::new(self.k, tables, padding)?
            .into_iter()
            .enumerate()
            .skip(skip)
            .peekable();
        let (index, circuit) = slices
            .next()
            .expect("Expected exactly one slice, but found none.");
        assert!(
            slices.peek().is_none(),
            "Expected exactly one slice, but found more."
        );

        if mock_test {
            println!("mock test for slice {}...", index);
            circuit.mock_test(instances.clone())?;
        }

        type GeminiKzg = multilinear::Gemini<univariate::UnivariateKzg<PBN256>>;
        type HyperPlonk = backend::hyperplonk::HyperPlonk<GeminiKzg>;

        let circuit = match circuit {
            ZkWasmCircuit::Ongoing(_) => unimplemented!(),
            ZkWasmCircuit::LastSliceCircuit(circuit) => circuit,
        };
        let zkcircuit = get_zkwasm_circuit::<HyperPlonk, Bn256, _>(
            self.k,
            std::slice::from_ref(&circuit),
            instances,
        );

        let circuit_info = zkcircuit.circuit_info().unwrap();
        let instances = zkcircuit.instances.clone();

        let timer = start_timer(|| format!("setup-{}", self.k));
        let param =
            HyperPlonk::setup(&circuit_info, StdRng::from_seed(Default::default())).unwrap();
        end_timer(timer);

        let timer = start_timer(|| format!("preprocess-{}", self.k));
        let (pp, vp) = HyperPlonk::preprocess(&param, &circuit_info).unwrap();
        end_timer(timer);

        let _timer = start_timer(|| format!("prove-{}", self.k));
        let mut transcript = Keccak256Transcript::default();
        HyperPlonk::prove(
            &pp,
            &zkcircuit,
            &mut transcript,
            StdRng::from_seed(Default::default()),
        )
        .unwrap();
        let proof = transcript.into_proof();

        let _timer = start_timer(|| format!("verify-{}", self.k));
        let mut transcript = Keccak256Transcript::from_proof((), proof.as_slice());
        match HyperPlonk::verify(
            &vp,
            instances.as_slice(),
            &mut transcript,
            StdRng::from_seed(Default::default()),
        ) {
            Ok(_) => {
                println!("✅ Proof verification succeeded");
            }
            Err(err) => {
                panic!("❌ Proof verification failed: {:?}", err);
            }
        }

        Ok(())
    }
}
