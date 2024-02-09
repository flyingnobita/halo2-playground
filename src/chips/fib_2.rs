use halo2_proofs::{arithmetic::Field, circuit::*, plonk::*, poly::Rotation};
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct FibonacciConfig {
    pub advice: [Column<Advice>; 2],
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
        advice: [Column<Advice>; 2],
        instance: Column<Instance>,
    ) -> FibonacciConfig {
        let col_a = advice[0];
        let col_b = advice[1];
        let selector = meta.selector();

        meta.enable_equality(col_a);
        meta.enable_equality(col_b);
        meta.enable_equality(instance);

        // create a gate to enforce a constraint
        meta.create_gate("fibonacci", |meta| {
            // col_a | col_b | selector
            //   a       b        s
            //           c
            let a = meta.query_advice(col_a, Rotation::cur());
            let b = meta.query_advice(col_b, Rotation::cur());
            let c = meta.query_advice(col_b, Rotation::next());
            let s = meta.query_selector(selector);
            vec![s * (a + b - c)]
        });

        FibonacciConfig {
            advice: [col_a, col_b],
            instance,
            selector,
        }
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        nrows: usize,
    ) -> Result<AssignedCell<F, F>, Error> {
        layouter.assign_region(
            || "entire fibonacci table",
            |mut region| {
                self.config.selector.enable(&mut region, 0)?;

                let mut a_cell = region.assign_advice_from_instance(
                    || "f(0)",
                    self.config.instance,
                    0, // absolute row index of the instance column i.e. the `a` in `vec![a, b, out]`
                    self.config.advice[0],
                    0,
                )?;

                let mut b_cell = region.assign_advice_from_instance(
                    || "f(1)",
                    self.config.instance,
                    1, // absolute row index of the instance column i.e. the `b` in `vec![a, b, out]`
                    self.config.advice[1],
                    0,
                )?;

                for row in 1..nrows {
                    if row < nrows - 1 {
                        self.config.selector.enable(&mut region, row)?;
                    }

                    let c_cell = region.assign_advice(
                        || "advice",
                        self.config.advice[1],
                        row,
                        || a_cell.value().copied() + b_cell.value(),
                    )?;

                    a_cell = region.assign_advice(
                        || "a",
                        self.config.advice[0],
                        row,
                        || b_cell.value().copied(),
                    )?;

                    b_cell = c_cell;
                }

                Ok(b_cell)
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
