use ff::PrimeFieldBits;
use halo2_proofs_zcash::{
    arithmetic::FieldExt,
    circuit::{floor_planner::V1, Layouter, Value},
    plonk::{Assigned, Circuit, ConstraintSystem, Error},
};

use crate::chips::range_check_4::RangeCheckDecomposeConfig;

#[derive(Clone)]

struct RangeCheckDecomposeCircuit<F: FieldExt, const NUM_BITS: usize, const RANGE: usize> {
    value: Value<Assigned<F>>,
    num_bits: usize,
}

impl<F: FieldExt + PrimeFieldBits, const NUM_BITS: usize, const RANGE: usize> Circuit<F>
    for RangeCheckDecomposeCircuit<F, NUM_BITS, RANGE>
{
    type Config = RangeCheckDecomposeConfig<F, NUM_BITS, RANGE>;
    type FloorPlanner = V1;

    fn without_witnesses(&self) -> Self {
        Self {
            value: Value::unknown(),
            num_bits: self.num_bits,
        }
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        // Fixed column for constants
        let constants = meta.fixed_column();
        meta.enable_constant(constants);

        let value = meta.advice_column();
        RangeCheckDecomposeConfig::configure(meta, value)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        config.table.load(&mut layouter)?;

        // Witness the value somewhere
        let value = layouter.assign_region(
            || "Witness value",
            |mut region| {
                region.assign_advice(|| "Witness value", config.running_sum, 0, || self.value)
            },
        )?;

        config.assign(
            layouter.namespace(|| "Decompose value"),
            value,
            self.num_bits,
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use halo2_proofs_zcash::{circuit::Value, dev::MockProver, pasta::Fp, plonk::Assigned};
    use rand;

    use crate::circuits::range_check_4::RangeCheckDecomposeCircuit;

    #[test]
    fn test_decompose_4() {
        let k = 11;
        const NUM_BITS: usize = 10;
        const RANGE: usize = 1024; // 10-bit value

        // Random u64 value
        let value: u64 = rand::random();
        let value = Value::known(Assigned::from(Fp::from(value)));

        let circuit = RangeCheckDecomposeCircuit::<Fp, NUM_BITS, RANGE> {
            value,
            num_bits: 64,
        };

        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        prover.assert_satisfied();
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn print_decompose_4() {
        use plotters::{backend::BitMapBackend, drawing::IntoDrawingArea, style::WHITE};
        use std::path::Path;

        let filename = Path::new("./devgraphs/range-check-4-layout.png");
        let title = "Range Check 4 Layout";

        const K: u32 = 11;
        const LOOKUP_TABLE_RANGE: usize = 1024; // 10-bit value
        const NUM_BITS: usize = 10; // 10-bit value

        let root = BitMapBackend::new(filename, (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled(title, ("sans-serif", 60)).unwrap();

        let circuit = RangeCheckDecomposeCircuit::<Fp, NUM_BITS, LOOKUP_TABLE_RANGE> {
            value: Value::unknown(),
            num_bits: NUM_BITS,
        };
        halo2_proofs_zcash::dev::CircuitLayout::default()
            .render(K, &circuit, &root)
            .unwrap();
    }
}
