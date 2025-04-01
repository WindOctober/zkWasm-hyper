use halo2_proofs::arithmetic::PrimeField;
use halo2_proofs::circuit::Layouter;
use halo2_proofs::circuit::Value;
use halo2_proofs::plonk::Error;
use specs::external_host_call_table::ExternalHostCallTable;

use super::ExternalHostCallChip;

impl<F: PrimeField> ExternalHostCallChip<F> {
    pub(in crate::circuits) fn assign(
        self,
        mut layouter: impl Layouter<F>,
        table: &ExternalHostCallTable,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "foreign table",
            |mut region| {
                // Assign Fixed Column
                {
                    for offset in 0..self.maximal_available_rows {
                        region.assign_fixed(
                            || "external host call idx",
                            self.config.idx,
                            offset,
                            || Value::known(F::from(offset as u64)),
                        )?;
                    }
                }

                // Assign Advice Columns
                {
                    let mut offset = 0;

                    {
                        region.assign_advice(
                            || "external host call opcode",
                            self.config.opcode,
                            offset,
                            || Value::known(F::ZERO),
                        )?;

                        region.assign_advice(
                            || "external host call operand",
                            self.config.operand,
                            offset,
                            || Value::known(F::ZERO),
                        )?;
                    }

                    offset += 1;

                    for entry in table.entries() {
                        region.assign_advice(
                            || "external host call opcode",
                            self.config.opcode,
                            offset,
                            || Value::known(F::from(entry.op as u64)),
                        )?;

                        region.assign_advice(
                            || "external host call operand",
                            self.config.operand,
                            offset,
                            || Value::known(F::from(entry.value)),
                        )?;

                        offset += 1;
                    }
                }

                Ok(())
            },
        )?;

        Ok(())
    }
}
