// Created on: 1992-06-24
// Created by: Gilles DEBARBOUILLE
// Copyright (c) 1992-1999 Matra Datavision
// Copyright (c) 1999-2014 OPEN CASCADE SAS
//
// This file is part of Open CASCADE Technology software library.
//
// This library is free software; you can redistribute it and/or modify it under
// the terms of the GNU Lesser General Public License version 2.1 as published
// by the Free Software Foundation, with special exception defined in the file
// OCCT_LGPL_EXCEPTION.txt. Consult the file LICENSE_LGPL_21.txt included in OCCT
// distribution for complete text of the license and disclaimer of any warranty.
//
// Alternatively, this file may be used under the terms of Open CASCADE
// commercial license or contractual agreement.

use super::{Failure, QuantityDefinition, UnitDefinition, UnitsDictionary, lexicon};
use crate::tcollection::AsciiString;
use crate::units::evaluate;
type ConversionMatrix = [[f64; 50]; 50];
impl UnitsDictionary {
    pub(super) fn load() -> Result<Self, Failure> {
        let mut dictionary = Self {
            lexicon: lexicon::formula(),
            quantities: Vec::new(),
        };
        let lines: Vec<_> = include_bytes!("../Units.dat")
            .split(|byte| *byte == b'\n')
            .collect();
        let mut matrix = [[0.0; 50]; 50];
        let mut position = 0;
        while position < lines.len() {
            let line = lines[position].trim_ascii_end();
            position += 1;
            if line.is_empty() {
                continue;
            }
            if line[0] == b'.' {
                dictionary.finish(&mut matrix)?;
                dictionary
                    .quantities
                    .push(quantity_header(lines[position + 1]));
                matrix = [[0.0; 50]; 50];
                position += 3;
            } else {
                dictionary.load_unit(line, &mut matrix)?;
            }
        }
        Ok(dictionary)
    }
    fn load_unit(&mut self, line: &[u8], matrix: &mut ConversionMatrix) -> Result<(), Failure> {
        let name = field(line, 0, 51);
        let symbol = field(line, 51, 27);
        let conversion = field(line, 78, 27);
        let base = field(line, 105, 27);
        if name.is_empty() && symbol.is_empty() && conversion.is_empty() && base.is_empty() {
            return Ok(());
        }
        let shifted = conversion.first() == Some(&b'[');
        let coefficient = coefficient(conversion, shifted)?;
        let quantity = self.quantities.last_mut().unwrap();
        if !name.is_empty() {
            quantity.units.push(UnitDefinition {
                symbols: Vec::new(),
                value: 0.0,
                shift: 0.0,
            });
        }
        let unit = quantity.units.last_mut().unwrap();
        if shifted {
            unit.shift = coefficient;
        }
        if !symbol.is_empty() {
            unit.symbols.push(symbol.to_vec());
            lexicon::add_unit(&mut self.lexicon, symbol);
        }
        self.link_conversion(base, if shifted { 1.0 } else { coefficient }, matrix)
    }
    fn link_conversion(
        &mut self,
        base: &[u8],
        coefficient: f64,
        matrix: &mut ConversionMatrix,
    ) -> Result<(), Failure> {
        let quantity = self.quantities.last().unwrap();
        let number = quantity.units.len();
        if base.is_empty() {
            if number == 1 {
                matrix[1][0] = coefficient;
                self.quantities.last_mut().unwrap().units[0].value = coefficient;
            }
            return Ok(());
        }
        let reference = quantity
            .units
            .iter()
            .position(|unit| unit.symbols.iter().any(|symbol| symbol == base))
            .map(|index| index + 1);
        if let Some(index) = reference.filter(|index| *index < number) {
            matrix[number][index] = coefficient;
        } else {
            matrix[number][0] = coefficient
                * self
                    .evaluate(base)?
                    .ok_or_else(|| {
                        Failure::new(&format!(
                            "Units dictionary base {:?}; expected evaluable unit",
                            String::from_utf8_lossy(base)
                        ))
                    })?
                    .value;
        }
        Ok(())
    }
    fn finish(&mut self, matrix: &mut ConversionMatrix) -> Result<(), Failure> {
        let Some(quantity) = self.quantities.last_mut() else {
            return Ok(());
        };
        let count = quantity.units.len();
        if count == 0 {
            return Ok(());
        }
        for (index, row) in matrix.iter_mut().enumerate().take(count + 1) {
            row[index] = 1.0;
        }
        for _ in 0..count {
            if (0..=count).all(|index| matrix[index][0] != 0.0) {
                break;
            }
            propagate(matrix, count);
        }
        if (0..=count).any(|index| matrix[index][0] == 0.0) {
            return Err(Failure::new(
                "Units dictionary: expected connected conversion graph",
            ));
        }
        for (index, unit) in quantity.units.iter_mut().enumerate() {
            unit.value = matrix[index + 1][0];
        }
        Ok(())
    }
}
fn field(line: &[u8], start: usize, width: usize) -> &[u8] {
    line.get(start..start.saturating_add(width).min(line.len()))
        .unwrap_or_default()
        .trim_ascii_end()
}
fn quantity_header(line: &[u8]) -> QuantityDefinition {
    let dimensions = std::array::from_fn(|index| {
        AsciiString::from_bytes(field(line, 40 + index * 10, 10))
            .real_value()
            .unwrap_or(0.0)
    });
    QuantityDefinition {
        name: field(line, 0, 40).to_vec(),
        dimensions,
        units: Vec::new(),
    }
}
fn coefficient(bytes: &[u8], shifted: bool) -> Result<f64, Failure> {
    let value = if shifted {
        &bytes[1..bytes.len() - 1]
    } else {
        bytes
    };
    if value.is_empty() {
        return Ok(1.0);
    }
    if value[0] != b'(' {
        return Ok(AsciiString::from_bytes(value).real_value().unwrap_or(0.0));
    }
    let mut sequence = lexicon::tokenize(&lexicon::formula(), &value[1..value.len() - 1]);
    lexicon::constants(&mut sequence);
    evaluate::calculate(&mut sequence)?
        .map(|token| token.value)
        .ok_or_else(|| Failure::new("Units dictionary: expected numeric coefficient"))
}
fn propagate(matrix: &mut ConversionMatrix, count: usize) {
    for current in 1..=count {
        if matrix[current][0] != 0.0 {
            continue;
        }
        for earlier in 1..current {
            if matrix[current][earlier] != 0.0 && matrix[earlier][0] != 0.0 {
                matrix[current][0] = matrix[earlier][0] * matrix[current][earlier];
            }
        }
        for later in current + 1..=count {
            if matrix[later][current] != 0.0 && matrix[later][0] != 0.0 {
                matrix[current][0] = matrix[later][0] / matrix[later][current];
            }
        }
    }
}
