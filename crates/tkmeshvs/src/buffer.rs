// SPDX-License-Identifier: GPL-2.0-only
//! `MeshVS_Buffer` at OCCT 3d097a0: a scratch buffer of a byte size, on the stack up to
//! `MeshVS_BufSize` (30 doubles) natively, viewed as doubles, integers, points or raw bytes.
//!
//! The native conversion operators reinterpret one storage; the port keeps one byte storage
//! and reads or writes the typed values in native byte order, so the views alias exactly as
//! natively without unsafe reinterpretation. The storage starts zeroed (natively it is not
//! initialised).
//!
//! ```
//! use martensite_tkmeshvs::MeshVSBuffer;
//! use rustycad_tkmath::gp::Point3;
//! let mut buffer = MeshVSBuffer::new(3 * 8);
//! buffer.set_real(1, 2.5);
//! assert_eq!(buffer.point(0).y(), 2.5);
//! buffer.set_point(0, &Point3::new(1.0, 2.0, 3.0));
//! assert_eq!(buffer.real(2), 3.0);
//! ```
use rustycad_tkmath::gp::Point3;

/// `MeshVS_BufSize`: the size of the automatic storage, in bytes.
pub const MESHVS_BUF_SIZE: usize = 10 * 3 * std::mem::size_of::<f64>();

/// A typed view of a scratch byte storage; `MeshVSBuffer::new(24)`.
pub struct MeshVSBuffer {
    bytes: Vec<u8>,
    is_dynamic: bool,
}

impl MeshVSBuffer {
    /// `MeshVS_Buffer(theSize)`: at least `MESHVS_BUF_SIZE` bytes, dynamic above it.
    /// Example: `MeshVSBuffer::new(1000).is_dynamic()`.
    pub fn new(size: usize) -> Self {
        let is_dynamic = size > MESHVS_BUF_SIZE;
        Self {
            bytes: vec![0; size.max(MESHVS_BUF_SIZE)],
            is_dynamic,
        }
    }
    /// Whether the native buffer would allocate (`myDynData` set); `buffer.is_dynamic()`.
    pub fn is_dynamic(&self) -> bool {
        self.is_dynamic
    }
    /// `operator void*`: the raw storage; `buffer.as_bytes_mut()[0] = 1`.
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
    /// `operator double&` at an offset: the double at `index`; `buffer.real(0)`.
    pub fn real(&self, index: usize) -> f64 {
        f64::from_ne_bytes(self.chunk::<8>(index * 8))
    }
    /// Writes the double at `index`; `buffer.set_real(0, 1.5)`.
    pub fn set_real(&mut self, index: usize, value: f64) {
        self.bytes[index * 8..index * 8 + 8].copy_from_slice(&value.to_ne_bytes());
    }
    /// `operator int&` at an offset: the integer at `index`; `buffer.integer(0)`.
    pub fn integer(&self, index: usize) -> i32 {
        i32::from_ne_bytes(self.chunk::<4>(index * 4))
    }
    /// Writes the integer at `index`; `buffer.set_integer(0, 7)`.
    pub fn set_integer(&mut self, index: usize, value: i32) {
        self.bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_ne_bytes());
    }
    /// `operator gp_Pnt&` at an offset: the point of the doubles `3 * index ..`; `buffer.point(0)`.
    pub fn point(&self, index: usize) -> Point3 {
        Point3::new(
            self.real(3 * index),
            self.real(3 * index + 1),
            self.real(3 * index + 2),
        )
    }
    /// Writes the point at `index`; `buffer.set_point(0, &p)`.
    pub fn set_point(&mut self, index: usize, point: &Point3) {
        self.set_real(3 * index, point.x());
        self.set_real(3 * index + 1, point.y());
        self.set_real(3 * index + 2, point.z());
    }
    fn chunk<const N: usize>(&self, offset: usize) -> [u8; N] {
        let mut out = [0; N];
        out.copy_from_slice(&self.bytes[offset..offset + N]);
        out
    }
}
