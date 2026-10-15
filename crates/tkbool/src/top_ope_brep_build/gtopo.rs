// Rust translation of ModelingAlgorithms/TKBool/TopOpeBRepBuild/TopOpeBRepBuild_GTopo,
// TopOpeBRepBuild_GIter and TopOpeBRepBuild_GTool (.hxx/.cxx) at
// 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! The tables of the states kept by a topological operation (`GTopo`), their iteration
//! (`GIter`) and the tables of the fuse, cut and common operations (`GTool`).
use crate::top_ope_brep_ds::{TopOpeBRepDSConfig, print_config};
use rustycad_tkernel::standard::Failure;
use rustycad_tkg3d::top_abs::{self, ShapeEnum, State};
use std::io::{self, Write};

/// Standard_ProgramError of the tables.
fn program_error(message: &str) -> Failure {
    Failure::with_type("Standard_ProgramError", message, "TopOpeBRepBuild_GTopo")
}

/// The states kept by an operation (`TopOpeBRepBuild_GTopo`): a table of the states IN,
/// ON, OUT of a shape relative to the other, the shape types and the configurations;
/// `let g = TopOpeBRepBuildGTool::g_fus_unsh(ShapeEnum::Face, ShapeEnum::Face);`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TopOpeBRepBuildGTopo {
    mycases: [[bool; 3]; 3],
    myt1: ShapeEnum,
    myt2: ShapeEnum,
    my_config1: TopOpeBRepDSConfig,
    my_config2: TopOpeBRepDSConfig,
    my_reverse_force: bool,
    my_reverse_value: bool,
}

impl Default for TopOpeBRepBuildGTopo {
    fn default() -> Self {
        Self::new()
    }
}

impl TopOpeBRepBuildGTopo {
    /// `TopOpeBRepBuild_GTopo()`: `Reset()`; `TopOpeBRepBuildGTopo::new()`.
    pub fn new() -> Self {
        let mut g = Self {
            mycases: [[false; 3]; 3],
            myt1: ShapeEnum::Shape,
            myt2: ShapeEnum::Shape,
            my_config1: TopOpeBRepDSConfig::UnshGeometry,
            my_config2: TopOpeBRepDSConfig::UnshGeometry,
            my_reverse_force: false,
            my_reverse_value: false,
        };
        g.reset();
        g
    }

    /// `TopOpeBRepBuild_GTopo(ii, in, io, ni, nn, no, oi, on, oo, t1, t2, C1, C2)`;
    /// `TopOpeBRepBuildGTopo::with_table([[false; 3]; 3], ShapeEnum::Face, ShapeEnum::Face, c, c)`.
    pub fn with_table(
        cases: [[bool; 3]; 3],
        t1: ShapeEnum,
        t2: ShapeEnum,
        c1: TopOpeBRepDSConfig,
        c2: TopOpeBRepDSConfig,
    ) -> Self {
        let mut g = Self::new();
        g.set(cases);
        (g.myt1, g.myt2, g.my_config1, g.my_config2) = (t1, t2, c1, c2);
        g
    }

    /// `Reset()`: no state kept, SHAPE types, UNSHGEOMETRY configurations, no forced
    /// reversal; `g.reset()`.
    pub fn reset(&mut self) {
        (self.myt1, self.myt2) = (ShapeEnum::Shape, ShapeEnum::Shape);
        self.mycases = [[false; 3]; 3];
        (self.my_config1, self.my_config2) = (
            TopOpeBRepDSConfig::UnshGeometry,
            TopOpeBRepDSConfig::UnshGeometry,
        );
        (self.my_reverse_force, self.my_reverse_value) = (false, false);
    }

    /// `Set(ii, in, io, ni, nn, no, oi, on, oo)`: the table by rows IN, ON, OUT;
    /// `g.set([[false; 3]; 3])`.
    pub fn set(&mut self, cases: [[bool; 3]; 3]) {
        self.mycases = cases;
    }

    /// `Type(t1, t2)`; `let (t1, t2) = g.type_();`.
    pub fn type_(&self) -> (ShapeEnum, ShapeEnum) {
        (self.myt1, self.myt2)
    }

    /// `ChangeType(t1, t2)`; `g.change_type(ShapeEnum::Face, ShapeEnum::Edge)`.
    pub fn change_type(&mut self, t1: ShapeEnum, t2: ShapeEnum) {
        (self.myt1, self.myt2) = (t1, t2);
    }

    /// `Config1()`; `g.config1()`.
    pub fn config1(&self) -> TopOpeBRepDSConfig {
        self.my_config1
    }

    /// `Config2()`; `g.config2()`.
    pub fn config2(&self) -> TopOpeBRepDSConfig {
        self.my_config2
    }

    /// `ChangeConfig(C1, C2)`; `g.change_config(c1, c2)`.
    pub fn change_config(&mut self, c1: TopOpeBRepDSConfig, c2: TopOpeBRepDSConfig) {
        (self.my_config1, self.my_config2) = (c1, c2);
    }

    /// `Value(i1, i2)`: the cell of the indices 0..3 (panics out of range, natively
    /// unchecked); `g.value_at(0, 1)`.
    pub fn value_at(&self, i1: usize, i2: usize) -> bool {
        self.mycases[i1][i2]
    }

    /// `Value(s1, s2)`: the cell of the states IN, ON or OUT (Standard_ProgramError
    /// otherwise); `g.value(State::In, State::Out)?`.
    pub fn value(&self, s1: State, s2: State) -> Result<bool, Failure> {
        Ok(self.mycases[self.g_index(s1)?][self.g_index(s2)?])
    }

    /// `Value(II)`: the cell of the index 0..9 by rows (the first cell for another);
    /// `g.value_index(4)`.
    pub fn value_index(&self, ii: i32) -> bool {
        let (i1, i2) = self.index(ii);
        self.value_at(i1, i2)
    }

    /// `ChangeValue(i1, i2, b)`; `g.change_value_at(0, 1, true)`.
    pub fn change_value_at(&mut self, i1: usize, i2: usize, b: bool) {
        self.mycases[i1][i2] = b;
    }

    /// `ChangeValue(s1, s2, b)`; `g.change_value(State::In, State::Out, true)?`.
    pub fn change_value(&mut self, s1: State, s2: State, b: bool) -> Result<(), Failure> {
        let (i1, i2) = (self.g_index(s1)?, self.g_index(s2)?);
        self.mycases[i1][i2] = b;
        Ok(())
    }

    /// `GIndex(s)`: 0, 1, 2 for IN, ON, OUT; Standard_ProgramError otherwise;
    /// `g.g_index(State::On)?`.
    pub fn g_index(&self, s: State) -> Result<usize, Failure> {
        match s {
            State::In => Ok(0),
            State::On => Ok(1),
            State::Out => Ok(2),
            State::Unknown => Err(program_error("GIndex : bad input")),
        }
    }

    /// `GState(i)`: IN, ON, OUT for 0, 1, 2; Standard_ProgramError otherwise;
    /// `g.g_state(1)?`.
    pub fn g_state(&self, i: usize) -> Result<State, Failure> {
        match i {
            0 => Ok(State::In),
            1 => Ok(State::On),
            2 => Ok(State::Out),
            _ => Err(program_error("GState : bad input")),
        }
    }

    /// `Index(II, i1, i2)`: the row and column of the index 0..9 (natively the outputs
    /// stay as given otherwise: (0, 0) here); `g.index(5)`.
    pub fn index(&self, ii: i32) -> (usize, usize) {
        if (0..=8).contains(&ii) {
            ((ii / 3) as usize, (ii % 3) as usize)
        } else {
            (0, 0)
        }
    }

    /// `DumpVal(OS, s1, s2)`: the cell as 0 or 1; `g.dump_val(&mut out, State::In, State::In)?`.
    pub fn dump_val(&self, os: &mut impl Write, s1: State, s2: State) -> Result<(), Failure> {
        let v = self.value(s1, s2)?;
        write!(os, "{}", u8::from(v)).map_err(io_failure)
    }

    /// `DumpType(OS)`: the two types; `g.dump_type(&mut out)?`.
    pub fn dump_type(&self, os: &mut impl Write) -> io::Result<()> {
        top_abs::print_shape_type(self.myt1, os)?;
        write!(os, "/")?;
        top_abs::print_shape_type(self.myt2, os)
    }

    /// `DumpSSB(OS, s1, s2, b)` (static); `TopOpeBRepBuildGTopo::dump_ssb(&mut out, s1, s2, true)?`.
    pub fn dump_ssb(os: &mut impl Write, s1: State, s2: State, b: bool) -> io::Result<()> {
        top_abs::print_state(s1, os)?;
        write!(os, " ")?;
        top_abs::print_state(s2, os)?;
        write!(os, " : {}", u8::from(b))
    }

    /// One row of `Dump`.
    fn dump_row(
        &self,
        os: &mut impl Write,
        s: &str,
        label: &str,
        s1: State,
    ) -> Result<(), Failure> {
        write!(os, "{s}{label} ").map_err(io_failure)?;
        self.dump_val(os, s1, State::In)?;
        write!(os, " ").map_err(io_failure)?;
        self.dump_val(os, s1, State::On)?;
        write!(os, " ").map_err(io_failure)?;
        self.dump_val(os, s1, State::Out)?;
        writeln!(os).map_err(io_failure)
    }

    /// `Dump(OS, s)`: the types, configurations, forced reversal and the table, each
    /// row after the prefix `s`; `g.dump(&mut out, "")?`.
    pub fn dump(&self, os: &mut impl Write, s: &str) -> Result<(), Failure> {
        self.dump_type(os).map_err(io_failure)?;
        write!(os, " ").map_err(io_failure)?;
        print_config(self.my_config1, os).map_err(io_failure)?;
        write!(os, " ").map_err(io_failure)?;
        print_config(self.my_config2, os).map_err(io_failure)?;
        writeln!(os).map_err(io_failure)?;
        if self.my_reverse_force {
            writeln!(os, "reverse value : {}", u8::from(self.my_reverse_value))
                .map_err(io_failure)?;
        }
        writeln!(os, "{s}\\ I N O").map_err(io_failure)?;
        self.dump_row(os, s, "I", State::In)?;
        self.dump_row(os, s, "N", State::On)?;
        self.dump_row(os, s, "O", State::Out)
    }

    /// `StatesON(s1, s2)`: the states kept with ON (Standard_ProgramError when
    /// undecided); `let (s1, s2) = g.states_on()?;`.
    pub fn states_on(&self) -> Result<(State, State), Failure> {
        let mut s1 = State::Unknown;
        if self.value(State::On, State::In)? {
            s1 = State::In;
        } else if self.value(State::On, State::Out)? {
            s1 = State::Out;
        }
        let mut s2 = State::Unknown;
        if self.value(State::In, State::On)? {
            s2 = State::In;
        } else if self.value(State::Out, State::On)? {
            s2 = State::Out;
        }
        if s1 == State::Unknown || s2 == State::Unknown {
            return Err(program_error("Gtopo : StatesON incorrect"));
        }
        Ok((s1, s2))
    }

    /// `IsToReverse1()`: the forced value, else whether the first shape is kept IN
    /// (not both); `g.is_to_reverse1()?`.
    pub fn is_to_reverse1(&self) -> Result<bool, Failure> {
        if self.my_reverse_force {
            return Ok(self.my_reverse_value);
        }
        let (s1, s2) = self.states_on()?;
        Ok(!(s1 == State::In && s2 == State::In) && s1 == State::In)
    }

    /// `IsToReverse2()`; `g.is_to_reverse2()?`.
    pub fn is_to_reverse2(&self) -> Result<bool, Failure> {
        if self.my_reverse_force {
            return Ok(self.my_reverse_value);
        }
        let (s1, s2) = self.states_on()?;
        Ok(!(s1 == State::In && s2 == State::In) && s2 == State::In)
    }

    /// `SetReverse(rev)`: the forced reversal; `g.set_reverse(true)`.
    pub fn set_reverse(&mut self, rev: bool) {
        (self.my_reverse_force, self.my_reverse_value) = (true, rev);
    }

    /// `Reverse()`: the forced reversal; Standard_ProgramError when not forced;
    /// `g.reverse()?`.
    pub fn reverse(&self) -> Result<bool, Failure> {
        if self.my_reverse_force {
            return Ok(self.my_reverse_value);
        }
        Err(program_error("GTopo::ReverseValue undefined"))
    }

    /// `CopyPermuted()`: the table of the shapes exchanged; `g.copy_permuted()`.
    pub fn copy_permuted(&self) -> Self {
        let mut g = Self::new();
        g.change_type(self.myt2, self.myt1);
        g.change_config(self.my_config2, self.my_config1);
        for i in 0..3 {
            for j in 0..3 {
                g.change_value_at(j, i, self.value_at(i, j));
            }
        }
        if self.my_reverse_force {
            g.set_reverse(self.my_reverse_value);
        }
        g
    }
}

/// A Failure of a write.
pub(super) fn io_failure(e: std::io::Error) -> Failure {
    Failure::new(&e.to_string())
}
