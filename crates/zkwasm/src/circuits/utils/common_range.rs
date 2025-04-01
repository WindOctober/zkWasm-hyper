use super::Context;
use crate::circuits::rtable::RangeTableConfig;
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
pub struct CommonRangeColumn<F: PrimeField> {
    pub col: Column<Advice>,
    _mark: PhantomData<F>,
}

impl<F: PrimeField> CommonRangeColumn<F> {
    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        cols: &mut impl Iterator<Item = Column<Advice>>,
        rtable: &RangeTableConfig<F>,
        enable: impl Fn(&mut VirtualCells<'_, F>) -> Expression<F>,
    ) -> Self {
        let col = cols.next().unwrap();

        rtable.configure_in_common_range(meta, "common range", |meta| {
            curr!(meta, col) * enable(meta)
        });

        Self {
            col,
            _mark: PhantomData,
        }
    }

    pub fn assign(&self, ctx: &mut Context<F>, value: u64) -> Result<(), Error> {
        ctx.region.assign_advice(
            || "common range value",
            self.col,
            ctx.offset,
            || Value::known(F::from(value)),
        )?;

        Ok(())
    }
}
