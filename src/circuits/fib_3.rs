use halo2_proofs::{arithmetic::Field, circuit::*, plonk::*};
use std::marker::PhantomData;

use crate::chips::fib_3::{FibonacciChip, FibonacciConfig};

#[derive(Default)]
struct FibonacciCircuit3<F>(PhantomData<F>);

impl<F: Field> Circuit<F> for FibonacciCircuit3<F> {
    type Config = FibonacciConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advice = meta.advice_column();
        let instance = meta.instance_column();

        FibonacciChip::configure(meta, advice, instance)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = FibonacciChip::construct(config);

        let num_row = 10;
        let out_cell = chip.assign(layouter.namespace(|| "private out"), num_row)?;

        chip.expose_public(layouter.namespace(|| "out"), &out_cell, 2)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use halo2_proofs::dev::MockProver;
    use halo2_proofs::halo2curves::bn256::Fr as Fp;

    use crate::circuits::fib_3::FibonacciCircuit3;
    // use crate::circuits::utils::plot;

    #[test]
    fn fib_3() {
        let k = 4;

        let a = Fp::from(1); // F[1]
        let b = Fp::from(1); // F[2]
        let out = Fp::from(55); // F[10]

        let circuit = FibonacciCircuit3(PhantomData);

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
    fn plot_fib3() {
        use plotters::prelude::*;
        use std::path::Path;

        let filename = Path::new("./devgraphs/fib-3-layout.png");
        let title = "Fib 3 Layout";

        let root = BitMapBackend::new(filename, (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled(title, ("sans-serif", 60)).unwrap();

        let circuit = FibonacciCircuit3::<Fp>(PhantomData);
        halo2_proofs::dev::CircuitLayout::default()
            .render(4, &circuit, &root)
            .unwrap();
    }

    // #[cfg(feature = "dev-graph")]
    // #[test]
    // fn plot_fib3() {
    //     let circuit = FibonacciCircuit3::<Fp>(PhantomData);
    //     plot("fib-3", "Fib 3", circuit);
    // }
}
