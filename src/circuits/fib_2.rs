use halo2_proofs::{arithmetic::Field, circuit::*, plonk::*};
use std::marker::PhantomData;

use crate::chips::fib_2::{FibonacciChip, FibonacciConfig};

#[derive(Default)]
struct FibonacciCircuit2<F>(PhantomData<F>);

impl<F: Field> Circuit<F> for FibonacciCircuit2<F> {
    type Config = FibonacciConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let col_a = meta.advice_column();
        let col_b = meta.advice_column();
        let instance = meta.instance_column();

        FibonacciChip::configure(meta, [col_a, col_b], instance)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = FibonacciChip::construct(config);

        let num_row = 9;
        let out_cell = chip.assign(layouter.namespace(|| "private out"), num_row)?;

        chip.expose_public(layouter.namespace(|| "out"), &out_cell, 2)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::FibonacciCircuit2;
    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr as Fp;

    #[test]
    fn fib_2() {
        let k = 4;

        let a = Fp::from(1); // F[1]
        let b = Fp::from(1); // F[2]
        let out = Fp::from(55); // F[10]

        let circuit = FibonacciCircuit2(PhantomData);

        let mut public_input = vec![a, b, out];

        let prover = MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
        prover.assert_satisfied();

        public_input[2] += Fp::one();
        let _prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
        // uncomment the following line and the assert will fail
        // _prover.assert_satisfied();
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn plot_fib2() {
        use plotters::prelude::*;
        use std::path::Path;

        let filename = Path::new("./devgraphs/fib-2-layout.png");
        let title = "Fib 2 Layout";

        let root = BitMapBackend::new(filename, (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled(title, ("sans-serif", 60)).unwrap();

        let circuit = FibonacciCircuit2::<Fp>(PhantomData);
        halo2_proofs::dev::CircuitLayout::default()
            .render(4, &circuit, &root)
            .unwrap();
    }
}
