use std::marker::PhantomData;

use halo2_proofs_zcash::{
    arithmetic::FieldExt,
    circuit::{Layouter, Value},
    plonk::{ConstraintSystem, Error, TableColumn},
};

// A lookup table of values of NUM_BITS length
// e.g. NUM_BITS = 8, values = [0..255]
#[derive(Clone, Debug)]
pub struct RangeCheckTableConfig<F: FieldExt, const LOOKUP_TABLE_RANGE: usize> {
    pub(super) value: TableColumn,
    _marker: PhantomData<F>,
}

impl<F: FieldExt, const LOOKUP_TABLE_RANGE: usize> RangeCheckTableConfig<F, LOOKUP_TABLE_RANGE> {
    pub(super) fn configure(meta: &mut ConstraintSystem<F>) -> Self {
        let value = meta.lookup_table_column();

        Self {
            value,
            _marker: PhantomData,
        }
    }

    pub fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        layouter.assign_table(
            || "load range-check table",
            |mut table| {
                for (offset, value) in (0..LOOKUP_TABLE_RANGE).enumerate() {
                    table.assign_cell(
                        || "num_bits",
                        self.value,
                        offset,
                        || Value::known(F::from(value as u64)),
                    )?;
                }

                Ok(())
            },
        )
    }
}
