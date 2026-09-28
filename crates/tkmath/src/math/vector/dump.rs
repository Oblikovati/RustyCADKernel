// Copyright (c) 1997-1999 Matra Datavision
// Copyright (c) 1999-2023 OPEN CASCADE SAS
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

use super::{VectorBase, VectorItem};
use rustycad_tkernel::ncollection::Array1;
use std::fmt;
impl<T: VectorItem> VectorBase<'_, T> {
    /// Writes the native `Dump` text, one declared index per line; `vector.dump(&mut text)?`.
    ///
    /// Both instantiations print the `math_Vector` label, as the shared template does.
    pub fn dump(&self, stream: &mut impl fmt::Write) -> fmt::Result {
        writeln!(stream, "math_Vector of Length = {}", self.length())?;
        for index in self.lower()..=self.upper() {
            writeln!(
                stream,
                "math_Vector({index}) = {}",
                self[index].stream_text()
            )?;
        }
        Ok(())
    }
    /// Views the elements as the underlying bounded array; `vector.array1().upper()`.
    pub fn array1(&self) -> Array1<'_, T> {
        Array1::from_shared(self.values(), self.lower())
    }
}
