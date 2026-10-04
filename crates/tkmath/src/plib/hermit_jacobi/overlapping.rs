//! Preserve OCCT's row-by-row publication when caller output arrays overlap.
use super::HermitJacobi;
use crate::plib::access::validate_derivative_order;
use crate::plib::jacobi_polynomial::BasisRows;
use rustycad_tkernel::standard::Failure;

impl HermitJacobi {
    /// Evaluates into possibly overlapping rows in one caller buffer.
    /// `basis.d0123_overlapping(3, 0.5, &mut buffer, [0, 0, 0, 0])?`.
    /// Each requested row spans `work_degree() + 1` elements from its start.
    pub fn d0123_overlapping(
        &self,
        native_order: i32,
        u: f64,
        storage: &mut [f64],
        row_starts: [usize; 4],
    ) -> Result<(), Failure> {
        validate_derivative_order("PLib_HermitJacobi::D0123", native_order)?;
        let order: usize = native_order as usize;
        let count: usize = self.work_degree() as usize + 1;
        validate_overlapping_rows(storage.len(), count, order, row_starts)?;
        let results: [Vec<f64>; 4] = self.separate_output_rows(native_order, u, count)?;
        // OCCT computes all intermediate values before writing rows 0, 1, 2, 3.
        for derivative in 0..=order {
            let start: usize = row_starts[derivative];
            storage[start..start + count].copy_from_slice(&results[derivative]);
        }
        Ok(())
    }

    fn separate_output_rows(
        &self,
        order: i32,
        u: f64,
        count: usize,
    ) -> Result<[Vec<f64>; 4], Failure> {
        let mut results: [Vec<f64>; 4] = std::array::from_fn(|_| vec![0.0; count]);
        let [value, d1, d2, d3] = &mut results;
        self.d0123(
            order,
            u,
            BasisRows {
                rows: [value, d1, d2, d3],
            },
        )?;
        Ok(results)
    }
}

fn validate_overlapping_rows(
    length: usize,
    count: usize,
    order: usize,
    starts: [usize; 4],
) -> Result<(), Failure> {
    for (derivative, start) in starts.iter().copied().enumerate().take(order + 1) {
        if start.checked_add(count).is_some_and(|end| end <= length) {
            continue;
        }
        return Err(Failure::out_of_range(&format!(
            "PLib_HermitJacobi::D0123 row {derivative} starts at {start}: expected {count} elements within buffer length {length}"
        )));
    }
    Ok(())
}
