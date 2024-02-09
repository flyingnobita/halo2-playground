use std::marker::PhantomData;

use halo2_proofs_zcash::{
    arithmetic::FieldExt,
    circuit::{AssignedCell, Layouter, Value},
    plonk::{Advice, Assigned, Column, ConstraintSystem, Constraints, Error, Expression, Selector},
    poly::Rotation,
};

use crate::chips::range_check_2_table::RangeCheckTableConfig;

/// A range-constrained value in the circuit produced by the RangeCheckConfig.
#[derive(Debug, Clone)]
pub struct RangeConstrained<F: FieldExt, const RANGE: usize>(AssignedCell<Assigned<F>, F>);

#[derive(Debug, Clone)]
pub struct RangeCheckConfig<F: FieldExt, const RANGE: usize, const LOOKUP_TABLE_RANGE: usize> {
    // We want to use simple lookup for smaller range, and lookup table for
    // larger range. Thus LOOKUP_TABLE_RANGE is bigger than RANGE
    value: Column<Advice>,
    q_range_check: Selector,
    q_lookup: Selector,
    pub table: RangeCheckTableConfig<F, LOOKUP_TABLE_RANGE>,
}

pub struct RangeCheckChip<F: FieldExt, const RANGE: usize, const LOOKUP_TABLE_RANGE: usize> {
    pub config: RangeCheckConfig<F, RANGE, LOOKUP_TABLE_RANGE>,
    pub _marker: PhantomData<F>,
}

impl<F: FieldExt, const RANGE: usize, const LOOKUP_TABLE_RANGE: usize>
    RangeCheckChip<F, RANGE, LOOKUP_TABLE_RANGE>
{
    pub fn construct(config: RangeCheckConfig<F, RANGE, LOOKUP_TABLE_RANGE>) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advice_col: Column<Advice>,
    ) -> RangeCheckConfig<F, RANGE, LOOKUP_TABLE_RANGE> {
        let q_range_check = meta.selector();
        let q_lookup = meta.complex_selector();

        let table = RangeCheckTableConfig::configure(meta);

        //  value | q_range_check   | q_lookup | table_value |
        //    v   |      1          |   0      |     0       |
        //    v'  |      0          |   1      |     1       |
        meta.create_gate("Range Check", |virtual_cells| {
            let q = virtual_cells.query_selector(q_range_check);
            let value = virtual_cells.query_advice(advice_col, Rotation::cur());

            // Given a range R and a value v, returns the expression
            // (v) * (1 - v) * (2 - v) * ... * (R - 1 - v)
            let range_check = (1..RANGE).fold(value.clone(), |expr, i| {
                expr * (Expression::Constant(F::from(i as u64)) - value.clone())
            });

            Constraints::with_selector(q, [("range check", range_check)])
        });

        // Lookup
        // Check that a value is contained within a lookup table of values 0..RANGE
        meta.lookup(|virtual_cells| {
            let q_lookup = virtual_cells.query_selector(q_lookup);
            let lookup_value = virtual_cells.query_advice(advice_col, Rotation::cur());

            vec![(q_lookup * lookup_value, table.value)]
        });

        RangeCheckConfig {
            value: advice_col,
            q_lookup,
            q_range_check,
            table,
        }
    }

    pub fn assign_simple(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>,
    ) -> Result<RangeConstrained<F, RANGE>, Error> {
        // do not support if the range is bigger than the lookup table range

        layouter.assign_region(
            || "Assign simple",
            |mut region| {
                let offset = 0;

                self.config.q_range_check.enable(&mut region, offset)?;

                region
                    .assign_advice(|| "value", self.config.value, offset, || value)
                    .map(RangeConstrained)
            },
        )
    }

    pub fn assign_lookup_table(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>,
    ) -> Result<RangeConstrained<F, RANGE>, Error> {
        layouter.assign_region(
            || "Assign lookup table",
            |mut region| {
                let offset = 0;

                self.config.q_lookup.enable(&mut region, offset)?;

                region
                    .assign_advice(|| "value", self.config.value, offset, || value)
                    .map(RangeConstrained)
            },
        )
    }
}
