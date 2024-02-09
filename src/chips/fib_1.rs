use halo2_proofs::{arithmetic::Field, circuit::*, plonk::*, poly::Rotation};
use std::marker::PhantomData;

type AdviceColumnType<F> = (AssignedCell<F, F>, AssignedCell<F, F>, AssignedCell<F, F>);

#[derive(Debug, Clone)]
pub struct FibonacciConfig {
    pub advice: [Column<Advice>; 3],
    pub instance: Column<Instance>,
    pub selector: Selector,
}

pub struct FibonacciChip<F: Field> {
    config: FibonacciConfig,
    _marker: PhantomData<F>,
}

impl<F: Field> FibonacciChip<F> {
    pub fn construct(config: FibonacciConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(
        meta: &mut ConstraintSystem<F>,
        advice: [Column<Advice>; 3],
        instance: Column<Instance>,
    ) -> FibonacciConfig {
        let col_a = advice[0];
        let col_b = advice[1];
        let col_c = advice[2];
        let selector = meta.selector();

        meta.enable_equality(col_a);
        meta.enable_equality(col_b);
        meta.enable_equality(col_c);
        meta.enable_equality(instance);

        // create a gate to enforce a constraint
        meta.create_gate("fibonacci", |meta| {
            // col_a | col_b | col_c | selector
            //   a      b        c       s
            //
            let s = meta.query_selector(selector);
            let a = meta.query_advice(col_a, Rotation::cur());
            let b = meta.query_advice(col_b, Rotation::cur());
            let c = meta.query_advice(col_c, Rotation::cur());
            vec![s * (a + b - c)]
        });

        FibonacciConfig {
            advice: [col_a, col_b, col_c],
            instance,
            selector,
        }
    }

    pub fn assign_first_row(
        &self,
        mut layouter: impl Layouter<F>,
    ) -> Result<AdviceColumnType<F>, Error> {
        layouter.assign_region(
            || "first row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;

                let a_cell = region.assign_advice_from_instance(
                    || "f(0)",
                    self.config.instance,
                    0,
                    self.config.advice[0],
                    0,
                )?;

                let b_cell = region.assign_advice_from_instance(
                    || "f(1)",
                    self.config.instance,
                    1,
                    self.config.advice[1],
                    0,
                )?;

                let c_cell = region.assign_advice(
                    || "f(0) + f(1)",
                    self.config.advice[2],
                    0,
                    || a_cell.value().copied() + b_cell.value(),
                )?;

                Ok((a_cell, b_cell, c_cell))
            },
        )
    }

    pub fn assign_row(
        &self,
        mut layouter: impl Layouter<F>,
        prev_b: &AssignedCell<F, F>,
        prev_c: &AssignedCell<F, F>,
    ) -> Result<AssignedCell<F, F>, Error> {
        layouter.assign_region(
            || "next row",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;

                prev_b.copy_advice(|| "a", &mut region, self.config.advice[0], 0)?;
                prev_c.copy_advice(|| "b", &mut region, self.config.advice[1], 0)?;

                let c_cell = region.assign_advice(
                    || "c",
                    self.config.advice[2],
                    0,
                    || prev_b.value().copied() + prev_c.value(),
                )?;

                Ok(c_cell)
            },
        )
    }

    // Ensure a cell is equal to the value in the instance column
    pub fn expose_public(
        &self,
        mut layouter: impl Layouter<F>,
        cell: &AssignedCell<F, F>,
        row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, row)
    }
}
