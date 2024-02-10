use halo2_proofs_zcash::{
    arithmetic::FieldExt,
    circuit::{floor_planner::V1, Layouter, Value},
    plonk::{Assigned, Circuit, ConstraintSystem, Error},
};

use crate::chips::range_check_3::{RangeCheckChip, RangeCheckConfig};

#[derive(Default)]
struct RangeCheckCircuit3<F: FieldExt, const LOOKUP_TABLE_RANGE: usize, const NUM_BITS: usize> {
    num_bits: Value<u8>,
    value: Value<Assigned<F>>,
}

impl<F: FieldExt, const LOOKUP_TABLE_RANGE: usize, const NUM_BITS: usize> Circuit<F>
    for RangeCheckCircuit3<F, LOOKUP_TABLE_RANGE, NUM_BITS>
{
    type Config = RangeCheckConfig<F, LOOKUP_TABLE_RANGE, NUM_BITS>;
    type FloorPlanner = V1;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(cs: &mut ConstraintSystem<F>) -> Self::Config {
        let num_bits_col = cs.advice_column();
        let value_col = cs.advice_column();

        RangeCheckChip::<F, LOOKUP_TABLE_RANGE, NUM_BITS>::configure(cs, value_col, num_bits_col)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        config.table.load(&mut layouter)?;

        let chip = RangeCheckChip::<F, LOOKUP_TABLE_RANGE, NUM_BITS>::construct(config);
        chip.assign_lookup_table(layouter.namespace(|| "value"), self.value, self.num_bits)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use halo2_proofs_zcash::{
        circuit::Value,
        dev::{FailureLocation, MockProver, VerifyFailure},
        pasta::Fp,
    };

    use crate::circuits::range_check_3::RangeCheckCircuit3;

    #[test]
    fn range_check_3() {
        let k = 9;
        const NUM_BITS: usize = 8; // 8-bit value
        const LOOKUP_RANGE: usize = 256; // 8-bit value

        // Test Success Range Check
        for num_bits_lookup in 1u8..=NUM_BITS.try_into().unwrap() {
            for value_lookup in (1 << (num_bits_lookup - 1))..(1 << num_bits_lookup) {
                let circuit = RangeCheckCircuit3::<Fp, LOOKUP_RANGE, NUM_BITS> {
                    num_bits: Value::known(num_bits_lookup),
                    value: Value::known(Fp::from(value_lookup as u64).into()),
                };

                let prover = MockProver::run(k, &circuit, vec![]).unwrap();
                prover.assert_satisfied();
            }
        }

        // Test Out-of-range `value = 8`, `lookup_range = 256`
        let circuit = RangeCheckCircuit3::<Fp, LOOKUP_RANGE, NUM_BITS> {
            num_bits: Value::known(NUM_BITS.try_into().unwrap()),
            value: Value::known(Fp::from(NUM_BITS as u64).into()),
        };
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_eq!(
            prover.verify(),
            Err(vec![VerifyFailure::Lookup {
                lookup_index: 0,
                location: FailureLocation::InRegion {
                    region: (1, "Assign lookup table").into(),
                    offset: 0
                }
            }])
        );
    }

    #[cfg(feature = "dev-graph-zcash")]
    #[test]
    fn plot_range_check_3() {
        use plotters::{backend::BitMapBackend, drawing::IntoDrawingArea, style::WHITE};
        use std::path::Path;

        let filename = Path::new("./devgraphs/range-check-3-layout.png");
        let title = "Range Check 3 Layout";

        const K: u32 = 9;
        const LOOKUP_TABLE_RANGE: usize = 256; // 8-bit value
        const NUM_BITS: usize = 8; // 8-bit value

        let root = BitMapBackend::new(filename, (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled(title, ("sans-serif", 60)).unwrap();

        let circuit = RangeCheckCircuit3::<Fp, LOOKUP_TABLE_RANGE, NUM_BITS> {
            num_bits: Value::unknown(),
            value: Value::unknown(),
        };
        halo2_proofs_zcash::dev::CircuitLayout::default()
            .render(K, &circuit, &root)
            .unwrap();
    }
}
