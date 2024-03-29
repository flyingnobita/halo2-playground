use ff::PrimeFieldBits;
use halo2_proofs_zcash::{arithmetic::FieldExt, circuit::*, plonk::*, poly::Rotation};
use std::marker::PhantomData;

use crate::chips::range_check_4_table::RangeTableConfig;

/// This gadget range-constrains an element witnessed in the circuit to be N bits.
///
/// Internally, this gadget uses the `range_check` helper, which provides a K-bit
/// lookup table.
///
/// Given an element `value`, we use a running sum to break it into K-bit chunks.
/// Assume for now that N | K, and define C = N / K.
///
///     value = [b_0, b_1, ..., b_{N-1}]   (little-endian)
///           = c_0 + 2^K * c_1  + 2^{2K} * c_2 + ... + 2^{(C-1)K} * c_{C-1}
///
/// Initialise the running sum at
///                                 value = z_0.
///
/// Consequent terms of the running sum are z_{i+1} = (z_i - c_i) * 2^{-K}:
///
///                           z_1 = (z_0 - c_0) * 2^{-K}
///                           z_2 = (z_1 - c_1) * 2^{-K}
///                              ...
///                       z_{C-1} = c_{C-1}
///                           z_C = (z_{C-1} - c_{C-1}) * 2^{-K}
///                               = 0
///
/// One configuration for this gadget could look like:
///
///     | running_sum |  q_decompose  |  q_partial_check   |  table_value  |
///     ---------------------------------------------------------------------
///     |     z_0     |       1       |          0          |       0       |
///     |     z_1     |       1       |          0          |       1       |
///     |     ...     |      ...      |         ...         |      ...      |
///     |   z_{C-1}   |       1       |          1          |      ...      |
///     |     z_C     |       0       |          0          |      ...      |
///
/// In the case where N is not a multiple of K, we have to handle a final chunk
/// that is `n` bits, where `n` < `K`. To do this:
///
/// - derive `z_C` from running sum
/// - witness `z_shifted` = z_C * 2^{K - n}
/// - assign a constant `shift` = 2^{-n}
///
/// - constrain:
///     - z_C * 2^K * shift = z_shifted
///
/// - lookup:
///     - z_C is in the range [0..K)
///     - z_shifted is in the range [0..K)  (no underflow)
///

#[derive(Debug, Clone)]
pub struct RangeCheckDecomposeConfig<
    F: FieldExt + PrimeFieldBits,
    const LOOKUP_NUM_BITS: usize,
    const LOOKUP_RANGE: usize,
> {
    // You'll need an advice column to witness your running sum;
    pub running_sum: Column<Advice>,
    // A selector to constrain the running sum;
    q_decompose: Selector,
    // A selector to handle the final partial chunk
    q_partial_check: Selector,
    // And of course, the K-bit lookup table
    pub table: RangeTableConfig<F, LOOKUP_NUM_BITS, LOOKUP_RANGE>,
    _marker: PhantomData<F>,
}

impl<F: FieldExt + PrimeFieldBits, const LOOKUP_NUM_BITS: usize, const LOOKUP_RANGE: usize>
    RangeCheckDecomposeConfig<F, LOOKUP_NUM_BITS, LOOKUP_RANGE>
{
    pub fn configure(meta: &mut ConstraintSystem<F>, running_sum: Column<Advice>) -> Self {
        // Create the needed columns and internal configs.
        let q_decompose = meta.complex_selector();
        let q_partial_check = meta.complex_selector();
        let table = RangeTableConfig::configure(meta);

        meta.enable_equality(running_sum);

        // Range-constrain each K-bit chunk `c_i = z_i - z_{i+1} * 2^K` derived from the running sum.
        meta.lookup(|meta| {
            let q_decompose = meta.query_selector(q_decompose);

            // z_i
            let z_cur = meta.query_advice(running_sum, Rotation::cur());
            // z_{i+1}
            let z_next = meta.query_advice(running_sum, Rotation::next());
            // c_i = z_i - z_{i+1} * 2^K
            let chunk = z_cur - z_next * F::from(1u64 << LOOKUP_NUM_BITS);

            // Lookup default value 0 when q_decompose = 0
            let not_q_decompose = Expression::Constant(F::one()) - q_decompose.clone();
            let default_chunk = Expression::Constant(F::zero());

            vec![(
                q_decompose * chunk + not_q_decompose * default_chunk,
                table.value,
            )]
        });

        // Handle the final partial chunk.
        meta.create_gate("final partial chunk", |meta| {
            let q_partial_check = meta.query_selector(q_partial_check);

            let final_chunk: Expression<F>;
            // derive final_chunk
            {
                // z_{C-1}
                let z_prev = meta.query_advice(running_sum, Rotation::prev());
                // z_C
                let z_cur = meta.query_advice(running_sum, Rotation::cur());
                // c_{C-1} = z_{C-1} - z_C * 2^K
                final_chunk = z_prev - z_cur * F::from(1u64 << LOOKUP_NUM_BITS);
            }

            // shifted_chunk = final_chunk * 2^{K - num_bits} where num_bits < K
            let shifted_chunk = meta.query_advice(running_sum, Rotation::next());

            // ensure shifted_chunk is derived correctly from final_chunk
            {
                // 2^{-num_bits} = 1 / 2^num_bits
                let inv_two_pow_s = meta.query_advice(running_sum, Rotation(2));
                let two_pow_k = F::from(1 << LOOKUP_NUM_BITS);
                // z_C = final_chunk * 2^K / 2^num_bits - final_chunk * 2^{K - num_bits}
                // z_C = c_{C-1} * 2^{K - num_bits} - c_{C-1} * 2^{K - num_bits} = 0
                let q_partial_check_expr = final_chunk * two_pow_k * inv_two_pow_s - shifted_chunk;
                Constraints::with_selector(q_partial_check, [q_partial_check_expr])
            }
        });

        // Lookup argument for shifted_chunk to ensure it is within K bits (i.e. in the range [0, 2^K))
        meta.lookup(|meta| {
            let q_partial_check = meta.query_selector(q_partial_check);
            let shifted = meta.query_advice(running_sum, Rotation::next());

            // Lookup default value 0 when q_partial_check = 0
            let not_q_partial_check = Expression::Constant(F::one()) - q_partial_check.clone();
            let default_chunk = Expression::Constant(F::zero());

            vec![(
                q_partial_check * shifted + not_q_partial_check * default_chunk,
                table.value,
            )]
        });

        Self {
            running_sum,
            q_decompose,
            q_partial_check,
            table,
            _marker: PhantomData,
        }
    }

    pub fn assign(
        &self,
        mut layouter: impl Layouter<F>,
        value: AssignedCell<Assigned<F>, F>,
        num_bits: usize,
    ) -> Result<(), Error> {
        let partial_len = num_bits % LOOKUP_NUM_BITS;

        layouter.assign_region(
            || "Decompose value",
            |mut region| {
                let mut offset = 0;

                // 0. Copy in the witnessed `value` at offset = 0
                let mut z = value.copy_advice(
                    || "Copy in value for decomposition",
                    &mut region,
                    self.running_sum,
                    offset,
                )?;

                // Increase offset after copying `value`
                offset += 1;

                // 1. Compute the interstitial running sum values {z_1, ..., z_C}}
                let expected_vec_len = if partial_len > 0 {
                    1 + num_bits / LOOKUP_NUM_BITS
                } else {
                    num_bits / LOOKUP_NUM_BITS
                };

                let running_sum: Vec<_> = value
                    .value()
                    .map(|&v| compute_running_sum::<_, LOOKUP_NUM_BITS>(v, num_bits))
                    .transpose_vec(expected_vec_len);

                // 2. Assign the running sum values
                for z_i in running_sum.into_iter() {
                    z = region.assign_advice(
                        || format!("assign z_{:?}", offset),
                        self.running_sum,
                        offset,
                        || z_i,
                    )?;
                    offset += 1;
                }

                // 3. Make sure to enable the relevant selector on each row of the running sum
                //    (but not on the row where z_C is witnessed)
                for offset in 0..(num_bits / LOOKUP_NUM_BITS) {
                    self.q_decompose.enable(&mut region, offset)?;
                }

                // 4. Constrain the final running sum `z_C` to be 0.
                region.constrain_constant(z.cell(), F::zero())?;

                // Handle partial chunk
                if partial_len > 0 {
                    // The final chunk
                    let final_chunk = value.value().map(|v| {
                        let v: Vec<_> = v
                            .evaluate()
                            .to_le_bits()
                            .iter()
                            .by_vals()
                            .take(num_bits)
                            .collect();
                        let final_chunk = &v[(num_bits - partial_len)..num_bits];
                        Assigned::from(F::from(lebs2ip(final_chunk)))
                    });
                    self.short_range_check(&mut region, offset - 1, final_chunk, partial_len)?;
                }

                Ok(())
            },
        )
    }

    /// Constrain `x` to be a partial_len word.
    ///
    /// q_partial_check is enabled on the offset of the final running sum z_C.
    ///
    ///     | running_sum             |  q_decompose  |  q_partial_check    |  table_value  |
    ///     ---------------------------------------------------------------------------------
    ///     | shifted = z_{C-1}       |       1       |          1          |      ...      |
    ///     | inv_two_pos_s = z_C     |       0       |          0          |      ...      |
    fn short_range_check(
        &self,
        region: &mut Region<'_, F>,
        offset: usize,
        element: Value<Assigned<F>>,
        partial_len: usize,
    ) -> Result<(), Error> {
        // Enable `q_partial_check`
        self.q_partial_check.enable(region, offset)?;

        // Assign shifted `element * 2^{K - partial_len}`
        let shifted = element.into_field() * F::from(1 << (LOOKUP_NUM_BITS - partial_len));
        region.assign_advice(
            || format!("element * 2^({}-{})", LOOKUP_NUM_BITS, partial_len),
            self.running_sum,
            offset + 1,
            || shifted,
        )?;

        // Assign 2^{-partial_len} from a fixed column.
        let inv_two_pow_s = F::from(1 << partial_len).invert().unwrap();
        region.assign_advice_from_constant(
            || format!("2^(-{})", partial_len),
            self.running_sum,
            offset + 2,
            inv_two_pow_s,
        )?;

        Ok(())
    }
}

// little-endian bits to u64
fn lebs2ip(bits: &[bool]) -> u64 {
    assert!(bits.len() <= 64);
    bits.iter()
        .enumerate()
        .fold(0u64, |acc, (i, b)| acc + if *b { 1 << i } else { 0 })
}

// Function to compute the interstitial running sum values {z_1, ..., z_C}}
fn compute_running_sum<F: FieldExt + PrimeFieldBits, const LOOKUP_NUM_BITS: usize>(
    value: Assigned<F>,
    num_bits: usize,
) -> Vec<Assigned<F>> {
    let mut running_sum = vec![];
    let mut z = value;

    // Get the little-endian bit representation of `value`.
    let value: Vec<_> = value
        .evaluate()
        .to_le_bits()
        .iter()
        .by_vals()
        .take(num_bits)
        .collect();
    for chunk in value.chunks(LOOKUP_NUM_BITS) {
        let chunk = Assigned::from(F::from(lebs2ip(chunk)));
        // z_{i+1} = (z_i - c_i) * 2^{-K}:
        z = (z - chunk) * Assigned::from(F::from(1u64 << LOOKUP_NUM_BITS)).invert();
        running_sum.push(z);
    }

    running_sum
}
