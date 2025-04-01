use halo2_proofs::arithmetic::PrimeField;
use halo2_proofs::plonk::Advice;
use halo2_proofs::plonk::Column;
use halo2_proofs::plonk::Fixed;
use std::marker::PhantomData;

mod assign;
mod configure;

#[derive(Clone)]
pub struct ExternalHostCallTableConfig<F: PrimeField> {
    idx: Column<Fixed>,
    opcode: Column<Advice>,
    operand: Column<Advice>,
    _phantom: PhantomData<F>,
}

pub struct ExternalHostCallChip<F: PrimeField> {
    config: ExternalHostCallTableConfig<F>,
    maximal_available_rows: usize,
}

impl<F: PrimeField> ExternalHostCallChip<F> {
    pub fn new(config: ExternalHostCallTableConfig<F>, maximal_available_rows: usize) -> Self {
        Self {
            config,
            maximal_available_rows,
        }
    }
}
