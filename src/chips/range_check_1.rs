use std::marker::PhantomData;

use halo2_proofs_zcash::{
    arithmetic::FieldExt,
    circuit::{Layouter, Value},
    plonk::{Advice, Assigned, Column, ConstraintSystem, Constraints, Error, Expression, Selector},
    poly::Rotation,
};

#[derive(Debug, Clone)]
pub struct RangeCheckConfig {
    value: Column<Advice>,
    selector: Selector,
}

pub struct RangeCheckChip<F: FieldExt, const RANGE: usize> {
    pub config: RangeCheckConfig,
    pub _marker: PhantomData<F>,
}

impl<F: FieldExt, const RANGE: usize> RangeCheckChip<F, RANGE> {
    pub fn construct(config: RangeCheckConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advice_column: Column<Advice>,
    ) -> RangeCheckConfig {
        let selector = meta.selector();

        //  value | selector
        //    v        s
        meta.create_gate("Range Check", |virtual_cells| {
            let q = virtual_cells.query_selector(selector);
            let value = virtual_cells.query_advice(advice_column, Rotation::cur());

            // Given a range R and a value v, returns the expression
            // (v) * (1 - v) * (2 - v) * ... * (R - 1 - v)
            let range_check = (1..RANGE).fold(value.clone(), |expr, i| {
                expr * (Expression::Constant(F::from(i as u64)) - value.clone())
            });

            Constraints::with_selector(q, [("range check", range_check)])
        });
        RangeCheckConfig {
            value: advice_column,
            selector,
        }
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        value: Value<Assigned<F>>,
    ) -> Result<(), Error> {
        layouter.assign_region(
            || "Assign value",
            |mut region| {
                let offset = 0;

                self.config.selector.enable(&mut region, offset)?;

                region.assign_advice(|| "value", self.config.value, offset, || value)
            },
        )?;
        Ok(())
    }
}
