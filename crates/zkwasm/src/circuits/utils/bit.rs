use super::Context;
use crate::constant_from;
use crate::curr;
use halo2_proofs::arithmetic::PrimeField;
use halo2_proofs::circuit::Value;
use halo2_proofs::plonk::Advice;
use halo2_proofs::plonk::Column;
use halo2_proofs::plonk::ConstraintSystem;
use halo2_proofs::plonk::Error;
use halo2_proofs::plonk::Expression;
use halo2_proofs::plonk::VirtualCells;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct BitColumn<F: PrimeField> {
    pub col: Column<Advice>,
    _mark: PhantomData<F>,
}

impl<F: PrimeField> BitColumn<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        enable: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) -> Self {
        let col = cols.next().unwrap();

        meta.create_gate("bit column", |meta| {
            vec![curr!(meta, col) * (constant_from!(1) - curr!(meta, col)) * enable(meta)]
        });

        Self {
            col,
            _mark: PhantomData,
        }
    }

    pub fn assign(&self, ctx: &mut Context<F>, value: bool) -> Result<(), Error> {
        ctx.region.assign_advice(
            || "bit value",
            self.col,
            ctx.offset,
            || Value::known(if value { F::ONE } else { F::ZERO }),
        )?;

        Ok(())
    }
}
