use halo2_proofs_zcash::{
    arithmetic::FieldExt,
    circuit::{Layouter, SimpleFloorPlanner, Value},
    plonk::{Assigned, Circuit, ConstraintSystem, Error},
};

use crate::chips::range_check_1::{RangeCheckChip, RangeCheckConfig};

#[derive(Default)]
struct RangeCheckCircuit1<F: FieldExt, const RANGE: usize> {
    value: Value<Assigned<F>>,
}

impl<F: FieldExt, const RANGE: usize> Circuit<F> for RangeCheckCircuit1<F, RANGE> {
    type Config = RangeCheckConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let advice = meta.advice_column();

        RangeCheckChip::<F, RANGE>::configure(meta, advice)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
        let chip = RangeCheckChip::<F, RANGE>::construct(config);
        chip.assign(layouter.namespace(|| "value"), self.value)?;
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

    use crate::circuits::range_check_1::RangeCheckCircuit1;

    #[test]
    fn range_check_1() {
        let k = 4;
        const RANGE: usize = 8; // 3-bit value
                                // let testvalue: u64 = 22;

        for i in 0..RANGE {
            let circuit = RangeCheckCircuit1::<Fp, RANGE> {
                value: Value::known(Fp::from(i as u64).into()),
            };

            let prover = MockProver::run(k, &circuit, vec![]).unwrap();
            prover.assert_satisfied();
        }

        let testvalue: u64 = 22;
        // Out-of-range `value = 8`
        let circuit = RangeCheckCircuit1::<Fp, RANGE> {
            value: Value::known(Fp::from(testvalue).into()),
        };
        let prover = MockProver::run(k, &circuit, vec![]).unwrap();
        assert_eq!(
            prover.verify(),
            Err(vec![VerifyFailure::ConstraintNotSatisfied {
                constraint: ((0, "Range Check").into(), 0, "range check").into(),
                location: FailureLocation::InRegion {
                    region: (0, "Assign value").into(),
                    offset: 0
                },
                cell_values: vec![(((Any::Advice, 0).into(), 0).into(), "0x16".to_string())]
            }])
        );
    }

    #[cfg(feature = "dev-graph-zcash")]
    #[test]
    fn plot_range_check_1() {
        use std::path::Path;

        use plotters::{backend::BitMapBackend, drawing::IntoDrawingArea, style::WHITE};

        let filename = Path::new("./devgraphs/range-check-1-layout.png");
        let title = "Range Check 1 Layout";

        let root = BitMapBackend::new(filename, (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled(title, ("sans-serif", 60)).unwrap();

        const RANGE: usize = 8; // 3-bit value
        let circuit = RangeCheckCircuit1::<Fp, RANGE> {
            value: Value::unknown(),
        };
        halo2_proofs_zcash::dev::CircuitLayout::default()
            .render(4, &circuit, &root)
            .unwrap();
    }
}
