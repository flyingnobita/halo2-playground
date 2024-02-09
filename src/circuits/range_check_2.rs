use halo2_proofs_zcash::{
    arithmetic::FieldExt,
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Assigned, Circuit, ConstraintSystem, Error},
};

use crate::chips::range_check_2::{RangeCheckChip, RangeCheckConfig};

#[derive(Default)]
struct RangeCheckCircuit2<F: FieldExt, const RANGE: usize, const LOOKUP_TABLE_RANGE: usize> {
    value: Value<Assigned<F>>,
    lookup_value: Value<Assigned<F>>,
}

impl<F: FieldExt, const RANGE: usize, const LOOKUP_TABLE_RANGE: usize> Circuit<F>
    for RangeCheckCircuit2<F, RANGE, LOOKUP_TABLE_RANGE>
{
    type Config = RangeCheckConfig<F, RANGE, LOOKUP_TABLE_RANGE>;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advice = meta.advice_column();

        RangeCheckChip::<F, RANGE, LOOKUP_TABLE_RANGE>::configure(meta, advice)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        config.table.load(&mut layouter)?;

        let chip = RangeCheckChip::<F, RANGE, LOOKUP_TABLE_RANGE>::construct(config);

        chip.assign_simple(layouter.namespace(|| "value"), self.value)?;

        chip.assign_lookup_table(layouter.namespace(|| "value"), self.lookup_value)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use halo2_proofs_zcash::{
        circuit::Value,
        dev::{FailureLocation, MockProver, VerifyFailure},
        pasta::Fp,
        plonk::Any,
    };

    use crate::circuits::range_check_2::RangeCheckCircuit2;

    #[test]
    fn range_check_2() {
        let k = 9;
        const RANGE: usize = 8; // 3-bit value
        const LOOKUP_RANGE: usize = 256; // 8-bit value

        // Test Success Range Check
        for i in 0..RANGE {
            for j in 0..LOOKUP_RANGE {
                let circuit = RangeCheckCircuit2::<Fp, RANGE, LOOKUP_RANGE> {
                    value: Value::known(Fp::from(i as u64).into()),
                    lookup_value: Value::known(Fp::from(j as u64).into()),
                };

                let prover = MockProver::run(k, &circuit, vec![]).unwrap();
                prover.assert_satisfied();
            }
        }

        // Test Out-of-range `value = 8`, `lookup_range = 256`
        {
            let circuit = RangeCheckCircuit2::<Fp, RANGE, LOOKUP_RANGE> {
                value: Value::known(Fp::from(RANGE as u64).into()),
                lookup_value: Value::known(Fp::from(LOOKUP_RANGE as u64).into()),
            };
            let prover = MockProver::run(k, &circuit, vec![]).unwrap();
            assert_eq!(
                prover.verify(),
                Err(vec![
                    VerifyFailure::ConstraintNotSatisfied {
                        constraint: ((0, "Range Check").into(), 0, "range check").into(),
                        location: FailureLocation::InRegion {
                            region: (1, "Assign simple").into(),
                            offset: 0
                        },
                        cell_values: vec![(((Any::Advice, 0).into(), 0).into(), "0x8".to_string())]
                    },
                    VerifyFailure::Lookup {
                        lookup_index: 0,
                        location: FailureLocation::InRegion {
                            region: (2, "Assign lookup table").into(),
                            offset: 0
                        }
                    }
                ])
            );
        }
    }

    #[cfg(feature = "dev-graph-zcash")]
    #[test]
    fn plot_range_check_2() {
        use std::path::Path;

        use plotters::{backend::BitMapBackend, drawing::IntoDrawingArea, style::WHITE};

        let filename = Path::new("./devgraphs/range-check-2-layout.png");
        let title = "Range Check 2 Layout";

        let root = BitMapBackend::new(filename, (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled(title, ("sans-serif", 60)).unwrap();

        const RANGE: usize = 8; // 3-bit value
        const LOOKUP_TABLE_RANGE: usize = 256; // 8-bit value
        let circuit = RangeCheckCircuit2::<Fp, RANGE, LOOKUP_TABLE_RANGE> {
            value: Value::unknown(),
            lookup_value: Value::unknown(),
        };
        halo2_proofs_zcash::dev::CircuitLayout::default()
            .render(4, &circuit, &root)
            .unwrap();
    }
}
