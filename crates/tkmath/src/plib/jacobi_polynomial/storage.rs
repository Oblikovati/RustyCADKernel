use rustycad_tkernel::standard::Failure;

pub(super) trait OutputRows {
    fn len(&self, row: usize) -> usize;
    fn read(&self, row: usize, index: usize) -> f64;
    fn write(&mut self, row: usize, index: usize, value: f64);
}

pub(super) struct SeparateRows<'a> {
    rows: [&'a mut [f64]; 4],
}

impl<'a> SeparateRows<'a> {
    pub(super) fn new(rows: [&'a mut [f64]; 4]) -> Self {
        Self { rows }
    }
}

impl OutputRows for SeparateRows<'_> {
    fn len(&self, row: usize) -> usize {
        self.rows[row].len()
    }

    fn read(&self, row: usize, index: usize) -> f64 {
        self.rows[row][index]
    }

    fn write(&mut self, row: usize, index: usize, value: f64) {
        self.rows[row][index] = value;
    }
}

pub(super) struct IndexedRows<'a> {
    values: &'a mut [f64],
    starts: [usize; 4],
}

impl<'a> IndexedRows<'a> {
    pub(super) fn new(
        values: &'a mut [f64],
        starts: [usize; 4],
        count: usize,
        last_row: usize,
    ) -> Result<Self, Failure> {
        validate_spans(values.len(), starts, count, last_row)?;
        Ok(Self { values, starts })
    }
}

impl OutputRows for IndexedRows<'_> {
    fn len(&self, row: usize) -> usize {
        self.values.len() - self.starts[row]
    }

    fn read(&self, row: usize, index: usize) -> f64 {
        self.values[self.starts[row] + index]
    }

    fn write(&mut self, row: usize, index: usize, value: f64) {
        self.values[self.starts[row] + index] = value;
    }
}

fn validate_spans(
    buffer_len: usize,
    starts: [usize; 4],
    count: usize,
    last_row: usize,
) -> Result<(), Failure> {
    for (row, start) in starts.iter().enumerate().take(last_row + 1) {
        let end: usize = start
            .checked_add(count)
            .ok_or_else(|| span_failure(row, *start, count, buffer_len))?;
        if end > buffer_len {
            return Err(span_failure(row, *start, count, buffer_len));
        }
    }
    Ok(())
}

fn span_failure(row: usize, start: usize, count: usize, buffer_len: usize) -> Failure {
    Failure::out_of_range(&format!(
        "PLib_JacobiPolynomial::D0123 row {row} starts {start}, needs {count} values within buffer length {buffer_len}"
    ))
}
