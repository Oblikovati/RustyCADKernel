//! `Units_UnitsDictionary::Creates`: the line-by-line reader of the compiled
//! `UnitsAPI_Units_dat` resource, including its conversion-matrix propagation.
use super::UnitsDictionary;
use crate::ncollection::HSequence;
use crate::standard::{Failure, Handle};
use crate::units::context::Units;
use crate::units::dimensions::Dimensions;
use crate::units::quantity::Quantity;
use crate::units::support::{
    c_string_to_real, item, length, new_sequence, object, right_adjust, scan_fields,
};
use crate::units::unit::{ShiftedUnit, Unit};
use crate::units::{MathSentence, UnitSentence};

mod matrix;
use matrix::ConversionMatrix;

/// The vendored `UnitsAPI_Units_dat` payload (see `tools/generate_units_data.py`).
const UNITS_DAT: &[u8] = include_bytes!("../Units.dat");

/// Every local of the native `Creates`, kept across lines exactly as C++ keeps them:
/// `coeff` and `move` in particular survive from one unit line to the next.
struct Reader<'a> {
    units: &'a Units,
    quantities: Handle<HSequence<Handle<Quantity>>>,
    section: Handle<HSequence<Handle<Unit>>>,
    quantity: Handle<Quantity>,
    matrix: ConversionMatrix,
    count: usize,
    coefficient: f64,
    movement: f64,
}

/// `readLine`: the next `'\n'`-terminated line; the text after the last newline (or
/// before a NUL) is never returned, like the native reader.
fn read_line(position: &mut Option<usize>) -> Vec<u8> {
    let Some(start) = *position else {
        return Vec::new();
    };
    let rest = &UNITS_DAT[start.min(UNITS_DAT.len())..];
    match rest.iter().position(|byte| *byte == b'\n' || *byte == 0) {
        Some(end) if rest[end] == b'\n' => {
            *position = Some(start + end + 1);
            rest[..end].to_vec()
        }
        _ => {
            *position = None;
            Vec::new()
        }
    }
}

impl UnitsDictionary {
    /// Rebuilds every quantity from the resource, adding each unit symbol to the
    /// context's units lexicon (`Units::LexiconUnits(false)->AddToken`), whose repeated
    /// meanings print to the context console; `dictionary.creates(&units)?`.
    pub fn creates(&self, units: &Units) -> Result<(), Failure> {
        let quantities = new_sequence::<Handle<Quantity>>();
        self.set_sequence(quantities.clone());
        let mut reader = Reader::new(units, quantities);
        let mut position = Some(0);
        while position.is_some() {
            let line = read_line(&mut position);
            if position.is_none() {
                break;
            }
            reader.line(&line, &mut position)?;
        }
        Ok(())
    }
}

impl<'a> Reader<'a> {
    fn new(units: &'a Units, quantities: Handle<HSequence<Handle<Quantity>>>) -> Self {
        Self {
            units,
            quantities,
            section: Handle::default(),
            quantity: Handle::default(),
            matrix: ConversionMatrix::new(),
            count: 0,
            coefficient: 0.0,
            movement: 0.0,
        }
    }
    fn line(&mut self, line: &[u8], position: &mut Option<usize>) -> Result<(), Failure> {
        let line = right_adjust(line);
        if line.is_empty() {
            return Ok(());
        }
        if line[0] != b'.' {
            return self.unit_line(line);
        }
        if self.count != 0 {
            self.finish_section()?;
        }
        read_line(position);
        let header = read_line(position);
        self.header(&header)?;
        read_line(position);
        Ok(())
    }
    /// Header: a 40-byte name and nine 10-byte exponents.
    fn header(&mut self, line: &[u8]) -> Result<(), Failure> {
        let fields = scan_fields(line, &[40, 10, 10, 10, 10, 10, 10, 10, 10, 10]);
        let exponents = std::array::from_fn(|i| c_string_to_real(fields[i + 1]));
        let dimensions = Dimensions::handle(exponents);
        self.count = 0;
        self.section = new_sequence::<Handle<Unit>>();
        let name = right_adjust(fields[0]);
        self.quantity = Handle::new(Quantity::new(name, &dimensions, &self.section));
        object(&self.quantities, "Units_UnitsDictionary::Creates")?.append(self.quantity.clone());
        self.matrix.clear();
        Ok(())
    }
    /// Unit line: 51-byte name, 27-byte symbol, factor and base unit.
    fn unit_line(&mut self, line: &[u8]) -> Result<(), Failure> {
        let fields: Vec<&[u8]> = scan_fields(line, &[51, 27, 27, 27])
            .into_iter()
            .map(right_adjust)
            .collect();
        let (name, symbol, base) = (fields[0], fields[1], fields[3]);
        if fields.iter().all(|field| field.is_empty()) {
            return Ok(());
        }
        let mut convert = fields[2].to_vec();
        let shifted = convert.first() == Some(&b'[');
        self.start_unit(name, shifted, &mut convert)?;
        if !symbol.is_empty() {
            self.add_symbol(symbol)?;
        }
        self.read_factor(shifted, &mut convert)?;
        if shifted && self.movement != 0.0 {
            self.apply_move()?;
        }
        self.link(base)
    }
    fn start_unit(
        &mut self,
        name: &[u8],
        shifted: bool,
        convert: &mut Vec<u8>,
    ) -> Result<(), Failure> {
        if shifted {
            self.coefficient = 1.0;
            convert.pop();
        }
        if name.is_empty() {
            return Ok(());
        }
        self.count += 1;
        let unit: Handle<Unit> = if shifted {
            Handle::new(ShiftedUnit::with_name(name)).cast()
        } else {
            Handle::new(Unit::with_name(name))
        };
        object(&unit, "Units_UnitsDictionary::Creates")?.set_quantity(&self.quantity);
        object(&self.section, "Units_UnitsDictionary::Creates")?.append(unit);
        Ok(())
    }
    fn last_unit(&self) -> Result<Handle<Unit>, Failure> {
        item(&self.section, length(&self.section)?)
    }
    fn add_symbol(&mut self, symbol: &[u8]) -> Result<(), Failure> {
        let lexicon = self.units.lexicon_units(false)?;
        let lexicon = object(&lexicon, "Units::LexiconUnits")?;
        lexicon.add_token(symbol, b"U", 0.0, &mut self.units.console())?;
        object(&self.last_unit()?, "Units_UnitsDictionary::Creates")?.symbol(symbol);
        Ok(())
    }
    fn read_factor(&mut self, shifted: bool, convert: &mut Vec<u8>) -> Result<(), Failure> {
        let start = usize::from(shifted);
        let value = if convert.get(start) == Some(&b'(') {
            convert.pop();
            let formula = convert.get(start + 1..).unwrap_or_default();
            Some(self.formula(formula)?)
        } else if !convert.is_empty() {
            Some(c_string_to_real(if shifted {
                &convert[start..]
            } else {
                convert
            }))
        } else {
            None
        };
        match (value, shifted) {
            (Some(value), true) => self.movement = value,
            (Some(value), false) => self.coefficient = value,
            (None, _) => self.coefficient = 1.0,
        }
        Ok(())
    }
    fn formula(&self, text: &[u8]) -> Result<f64, Failure> {
        let mut sentence = MathSentence::new(self.units, text)?;
        let token = sentence.evaluate()?;
        Ok(object(&token, "Units_UnitsDictionary::Creates")?.value())
    }
    fn apply_move(&self) -> Result<(), Failure> {
        let shifted = self.last_unit()?.cast::<ShiftedUnit>();
        object(&shifted, "Units_UnitsDictionary::Creates")?.set_movement(self.movement);
        Ok(())
    }
    fn link(&mut self, base: &[u8]) -> Result<(), Failure> {
        if base.is_empty() {
            if self.count == 1 {
                self.matrix.set(1, 0, self.coefficient)?;
                object(&item(&self.section, 1)?, "Units_UnitsDictionary::Creates")?
                    .set_value(self.coefficient);
            }
            return Ok(());
        }
        let found = self.find_unit(base)?;
        if found < self.count {
            return self.matrix.set(self.count, found, self.coefficient);
        }
        let mut sentence = UnitSentence::with_quantities(self.units, base, &self.quantities)?;
        let token = sentence.evaluate()?;
        let value = object(&token, "Units_UnitsDictionary::Creates")?.value();
        self.matrix.set(self.count, 0, self.coefficient * value)
    }
    /// One-based index of the first unit having `symbol`, or length + 1.
    fn find_unit(&self, symbol: &[u8]) -> Result<usize, Failure> {
        let total = length(&self.section)?;
        for index in 1..=total {
            if object(
                &item(&self.section, index)?,
                "Units_UnitsDictionary::Creates",
            )?
            .is_equal(symbol)?
            {
                return Ok(index);
            }
        }
        Ok(total + 1)
    }
    fn finish_section(&mut self) -> Result<(), Failure> {
        self.matrix.propagate(self.count)?;
        for index in 1..=length(&self.section)? {
            let value = self.matrix.get(index, 0)?;
            object(
                &item(&self.section, index)?,
                "Units_UnitsDictionary::Creates",
            )?
            .set_value(value);
        }
        Ok(())
    }
}
