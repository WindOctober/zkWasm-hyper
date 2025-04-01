use halo2_proofs::arithmetic::PrimeField;
use halo2_proofs::circuit::Layouter;
use halo2_proofs::circuit::Value;
use halo2_proofs::plonk::Error;

use super::ContextContHelperTableConfig;

pub struct ContextContHelperTableChip<F: PrimeField> {
    pub(crate) config: ContextContHelperTableConfig<F>,
}

impl<F: PrimeField> ContextContHelperTableChip<F> {
    pub fn new(config: ContextContHelperTableConfig<F>) -> Self {
        Self { config }
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        inputs: &[u64],
        outputs: &[u64],
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "context cont helper assign",
            |mut region| {
                for (offset, input) in inputs.iter().enumerate() {
                    region.assign_advice(
                        || "context cont input index",
                        self.config.input,
                        offset + 1, // The first fixed index should be 1.
                        || Value::known(F::from(*input)),
                    )?;
                }

                for (offset, output) in outputs.iter().enumerate() {
                    region.assign_advice(
                        || "context cont output index",
                        self.config.output,
                        offset + 1, // The first fixed index should be 1.
                        || Value::known(F::from(*output)),
                    )?;
                }

                Ok(())
            },
        )?;

        Ok(())
    }
}
