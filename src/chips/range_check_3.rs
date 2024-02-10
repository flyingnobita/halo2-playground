use std::marker::PhantomData;

use halo2_proofs_zcash::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, Value},
    plonk::{Advice, Assigned, Column, ConstraintSystem, Error, Expression, Selector},
    poly::Rotation,
};

use crate::chips::range_check_3_table::RangeCheckTableConfig;

/// A range-constrained value in the circuit produced by the RangeCheckConfig.
#[derive(Debug, Clone)]
pub struct RangeConstrained<F: FieldExt> {
    pub(crate) num_bits: AssignedCell<Assigned<F>, F>,
    pub(crate) assigned_cell: AssignedCell<Assigned<F>, F>,
}

#[derive(Debug, Clone)]
pub struct RangeCheckConfig<F: FieldExt, const LOOKUP_TABLE_RANGE: usize, const NUM_BITS: usize> {
    // We want to use simple lookup for smaller range, and lookup table for
    // larger range. Thus LOOKUP_TABLE_RANGE is bigger than RANGE
    q_lookup: Selector,
    num_bits_col: Column<Advice>,
    value_col: Column<Advice>,
    pub table: RangeCheckTableConfig<F, LOOKUP_TABLE_RANGE, NUM_BITS>,
}

pub struct RangeCheckChip<F: FieldExt, const LOOKUP_TABLE_RANGE: usize, const NUM_BITS: usize> {
    pub config: RangeCheckConfig<F, LOOKUP_TABLE_RANGE, NUM_BITS>,
    pub _marker: PhantomData<F>,
}

impl<F: FieldExt, const LOOKUP_TABLE_RANGE: usize, const NUM_BITS: usize>
    RangeCheckChip<F, LOOKUP_TABLE_RANGE, NUM_BITS>
{
    pub fn construct(config: RangeCheckConfig<F, LOOKUP_TABLE_RANGE, NUM_BITS>) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(
        cs: &mut ConstraintSystem<F>,
        num_bits_col: Column<Advice>,
        value_col: Column<Advice>,
    ) -> RangeCheckConfig<F, LOOKUP_TABLE_RANGE, NUM_BITS> {
        let q_lookup = cs.complex_selector();
        let table = RangeCheckTableConfig::configure(cs);

        // | lookup_value | q_lookup | table_num_bits | table_value |
        // | ------------ | -------- | -------------- | ----------- |
        // | v_0          | 0        | 1              | 0           |
        // | v_1          | 1        | 1              | 1           |
        // | v_2          | 1        | 2              | 2           |
        // | v_3          | 1        | 2              | 3           |
        // | ...          | ...      | 3              | 4           |
        cs.lookup(|virtual_cells| {
            let q_lookup_expr = virtual_cells.query_selector(q_lookup);
            let num_bits_expr = virtual_cells.query_advice(num_bits_col, Rotation::cur());
            let lookup_value_expr = virtual_cells.query_advice(value_col, Rotation::cur());

            // when q_lookup = 0, we want to lookup the default value of table_num_bits = 1, table_value = 0
            let not_q_lookup_expr = Expression::Constant(F::one()) - q_lookup_expr.clone();
            let default_num_bits_expr = Expression::Constant(F::one()); // 1-bit
            let default_value_expr = Expression::Constant(F::zero()); // 0 is a 1-bit value

            let num_bits_expr =
                q_lookup_expr.clone() * num_bits_expr + not_q_lookup_expr.clone() * default_num_bits_expr;
            let value_expr = q_lookup_expr * lookup_value_expr + not_q_lookup_expr * default_value_expr;

            // a vector of tuples (what we want to lookup, the corresponding table column to compare)
            vec![(num_bits_expr, table.num_bits), (value_expr, table.value)]
        });

        RangeCheckConfig {
            q_lookup,
            num_bits_col,
            value_col,
            table,
        }
    }

    pub fn assign_lookup_table(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>,
        num_bits: Value<u8>,
    ) -> Result<RangeConstrained<F>, Error> {
        layouter.assign_region(
            || "Assign lookup table",
            |mut region| {
                let offset = 0;

                self.config.q_lookup.enable(&mut region, offset)?;

                // Assign num_bits
                let num_bits_f = num_bits.map(|v| F::from(v as u64));
                let num_bits_assigned_cell = region.assign_advice(
                    || "num_bits",
                    self.config.num_bits_col,
                    offset,
                    || num_bits_f.into(),
                )?;

                let value_assigned_cell = region
                    .assign_advice(|| "value", self.config.value_col, offset, || value)
                    .unwrap();

                Ok(RangeConstrained {
                    num_bits: num_bits_assigned_cell,
                    assigned_cell: value_assigned_cell,
                })
            },
        )
    }
}
