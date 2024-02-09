use halo2_proofs::{arithmetic::Field, circuit::*, plonk::*};
use std::marker::PhantomData;

use crate::chips::fib_1::{FibonacciChip, FibonacciConfig};

#[derive(Default)]
struct FibonacciCircuit1<F>(PhantomData<F>);

impl<F: Field> Circuit<F> for FibonacciCircuit1<F> {
    type Config = FibonacciConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        // columns are defined inside the configure function in the circuit
        // layer, rather than the chip layer, this allows the columns to be
        // resued in different chips
        // selector column is not defined here because it will be optimized
        // by the selector combination in the backend
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let col_c = meta.advice_column();
        let instance = meta.instance_column();

        FibonacciChip::configure(meta, [col_a, col_b, col_c], instance)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = FibonacciChip::construct(config);

        let (_, mut prev_b, mut prev_c) =
            chip.assign_first_row(layouter.namespace(|| "first row"))?;

        for _i in 3..10 {
            let c_cell = chip.assign_row(layouter.namespace(|| "next row"), &prev_b, &prev_c)?;
            prev_b = prev_c;
            prev_c = c_cell;
        }

        chip.expose_public(layouter.namespace(|| "out"), &prev_c, 2)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::FibonacciCircuit1;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr as Fp;

    #[test]
    fn fib_1() {
        let k = 4;

        let a = Fp::from(1); // F[1]
        let b = Fp::from(1); // F[2]
        let out = Fp::from(55); // F[10]

        let circuit = FibonacciCircuit1(PhantomData);

        let mut public_input = vec![a, b, out];

        let prover = MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
        prover.assert_satisfied();

        public_input[2] += Fp::one();
        let _prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
        // _prover.assert_satisfied();
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn plot_fib1() {
        use plotters::prelude::*;
        use std::path::Path;

        let filename = Path::new("./devgraphs/fib-1-layout.png");
        let title = "Fib 1 Layout";

        let root = BitMapBackend::new(filename, (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled(title, ("sans-serif", 60)).unwrap();

        let circuit = FibonacciCircuit1::<Fp>(PhantomData);
        halo2_proofs::dev::CircuitLayout::default()
            .render(4, &circuit, &root)
            .unwrap();
    }
}
