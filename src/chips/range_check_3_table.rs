use std::marker::PhantomData;

use halo2_proofs_zcash::{
    arithmetic::FieldExt,
    circuit::{Layouter, Value},
    plonk::{ConstraintSystem, Error, TableColumn},
};

// A lookup table of values of NUM_BITS length
// e.g. NUM_BITS = 8, values = [0..255]
#[derive(Clone, Debug)]
pub struct RangeCheckTableConfig<
    F: FieldExt,
    const LOOKUP_TABLE_RANGE: usize,
    const NUM_BITS: usize,
> {
    pub(super) value: TableColumn,
    pub(super) num_bits: TableColumn,
    _marker: PhantomData<F>,
}

impl<F: FieldExt, const LOOKUP_TABLE_RANGE: usize, const NUM_BITS: usize>
    RangeCheckTableConfig<F, LOOKUP_TABLE_RANGE, NUM_BITS>
{
    pub(super) fn configure(cs: &mut ConstraintSystem<F>) -> Self {
        let value = cs.lookup_table_column();
        let num_bits = cs.lookup_table_column();

        Self {
            value,
            num_bits,
            _marker: PhantomData,
        }
    }

    pub fn load(&self, layouter: &mut impl Layouter<F>) -> Result<(), Error> {
        layouter.assign_table(
            || "load range-check table with num_bits",
            |mut table| {
                // | offset | num_bits | value |
                // | 0      | 1        | 0     |
                let mut offset = 0;
                {
                    table.assign_cell(
                        || "assign num_bits",
                        self.num_bits,
                        offset,
                        || Value::known(F::one()),
                    )?;
                    table.assign_cell(
                        || "assign value",
                        self.value,
                        offset,
                        || Value::known(F::zero()),
                    )?;
                    offset += 1;
                }

                // | offset | num_bits | value    |
                // | 0      | 1        | 0        |
                // | 1      | 1        | 1        | 1 << 0 = 1 .. 1 << 1 = 10
                // | 2      | 2        | 2 = 10   | 1 << 1 = 10 .. 1 << 2 = 100
                // | 3      | 2        | 3 = 11   |
                // | 4      | 3        | 4 = 100  | 1 << 2 = 100 .. 1 << 3 = 1000
                // | 5      | 3        | 5 = 101  |
                // | 6      | 3        | 5 = 110  |
                // | 7      | 3        | 5 = 111  |
                // ...
                for num_bits in 1..=NUM_BITS {
                    for value in (1 << (num_bits - 1))..(1 << num_bits) {
                        table.assign_cell(
                            || "assign num_bits",
                            self.num_bits,
                            offset,
                            || Value::known(F::from(num_bits as u64)),
                        )?;
                        table.assign_cell(
                            || "assign value",
                            self.value,
                            offset,
                            || Value::known(F::from(value as u64)),
                        )?;
                        offset += 1;
                    }
                }
                Ok(())
            },
        )
    }
}
