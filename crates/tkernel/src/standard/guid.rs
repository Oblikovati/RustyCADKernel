// Rust translation of FoundationClasses/TKernel/Standard/Standard_GUID.hxx and .cxx
// at 3d097a0328e71b826377d4814ab05ec3c3d23871.

//! OCCT `Standard_GUID` and `Standard_UUID`.
//!
//! The value is kept as its `Standard_UUID` fields; the C++ class stores `Data4[0..2]` as
//! one `char16_t` (`my16b3`), which matters only for the byte image hashed by
//! `std::hash<Standard_GUID>` ([`Guid::hash_code`]). The text forms live in `guid/text.rs`.

mod text;

use super::hashing::hash_bytes;

/// Number of ASCII characters in a GUID representation (`Standard_GUID_SIZE`).
pub const GUID_SIZE: usize = 36;
/// Number of ASCII/UTF-16 code units including the NUL terminator (`Standard_GUID_SIZE_ALLOC`).
pub const GUID_SIZE_ALLOC: usize = GUID_SIZE + 1;

/// Field layout of `Standard_UUID` (also usable in const contexts).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct Uuid {
    /// First 32 bits.
    pub data1: u32,
    /// Next 16 bits.
    pub data2: u16,
    /// Next 16 bits.
    pub data3: u16,
    /// Last eight bytes in displayed order.
    pub data4: [u8; 8],
}

/// A GUID with OCCT's strict 8-4-4-4-12 hexadecimal text format.
///
/// `operator==`/`operator!=` are `PartialEq`, the copy constructor and `operator=` are
/// `Copy`, and `std::hash<Standard_GUID>` is [`Guid::hash_code`] (also fed to `Hash`).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Guid(Uuid);

impl Guid {
    /// `Standard_GUID()`: the all-zero GUID.
    ///
    /// ```
    /// assert_eq!(rustycad_tkernel::standard::Guid::new().to_string(), "00000000-0000-0000-0000-000000000000");
    /// ```
    pub const fn new() -> Self {
        Self(Uuid {
            data1: 0,
            data2: 0,
            data3: 0,
            data4: [0; 8],
        })
    }

    /// `Standard_GUID(int, char16_t, char16_t, char16_t, uint8_t x 6)`: the ten components;
    /// the first field keeps its bit pattern and `d` supplies `Data4[0..2]` high byte first.
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// let guid = Guid::from_components(-1, 2, 3, 0x0405, 6, 7, 8, 9, 10, 11);
    /// assert_eq!(guid.to_string(), "ffffffff-0002-0003-0405-060708090a0b");
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub const fn from_components(
        a: i32,
        b: u16,
        c: u16,
        d: u16,
        e: u8,
        f: u8,
        g: u8,
        h: u8,
        i: u8,
        j: u8,
    ) -> Self {
        Self(Uuid {
            data1: a as u32,
            data2: b,
            data3: c,
            data4: [(d >> 8) as u8, d as u8, e, f, g, h, i, j],
        })
    }

    /// `Standard_GUID(const Standard_UUID&)`.
    ///
    /// ```
    /// use rustycad_tkernel::standard::{Guid, Uuid};
    /// assert_eq!(Guid::from_uuid(Uuid::default()), Guid::new());
    /// ```
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// `ToUUID()`: the UUID fields.
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// assert_eq!(Guid::new().to_uuid().data1, 0);
    /// ```
    pub const fn to_uuid(self) -> Uuid {
        self.0
    }

    /// `IsSame` (and `operator==` in const contexts, where the derived `==` is unavailable).
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// assert!(Guid::new().is_same(Guid::new()));
    /// ```
    pub const fn is_same(self, other: Self) -> bool {
        if self.0.data1 != other.0.data1
            || self.0.data2 != other.0.data2
            || self.0.data3 != other.0.data3
        {
            return false;
        }
        let mut i = 0;
        while i < 8 {
            if self.0.data4[i] != other.0.data4[i] {
                return false;
            }
            i += 1;
        }
        true
    }

    /// `IsNotSame` (and `operator!=` in const contexts).
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// assert!(!Guid::new().is_not_same(Guid::new()));
    /// ```
    pub const fn is_not_same(self, other: Self) -> bool {
        !self.is_same(other)
    }

    /// `Assign(const Standard_GUID&)` / `operator=`: copies another GUID.
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// let mut guid = Guid::new();
    /// guid.assign(Guid::from_components(1, 0, 0, 0, 0, 0, 0, 0, 0, 0));
    /// assert_eq!(guid.to_uuid().data1, 1);
    /// ```
    pub const fn assign(&mut self, other: Self) {
        *self = other;
    }

    /// Assigns a GUID and returns this value for chained assignment-style calls;
    /// `guid.assign_returning(other).assign_returning(Guid::new())`.
    pub const fn assign_returning(&mut self, other: Self) -> &mut Self {
        self.assign(other);
        self
    }

    /// `Assign(const Standard_UUID&)` / `operator=(const Standard_UUID&)`: copies UUID fields.
    ///
    /// ```
    /// use rustycad_tkernel::standard::{Guid, Uuid};
    /// let mut guid = Guid::from_components(1, 0, 0, 0, 0, 0, 0, 0, 0, 0);
    /// guid.assign_uuid(Uuid::default());
    /// assert_eq!(guid, Guid::new());
    /// ```
    pub const fn assign_uuid(&mut self, uuid: Uuid) {
        self.0 = uuid;
    }

    /// Assigns UUID fields and returns this GUID, matching native `operator=(Standard_UUID)`;
    /// `guid.assign_uuid_returning(uuid).assign_returning(other)`.
    pub const fn assign_uuid_returning(&mut self, uuid: Uuid) -> &mut Self {
        self.assign_uuid(uuid);
        self
    }

    /// The 16-byte image of the C++ fields in declaration order, native byte order.
    fn field_bytes(self) -> [u8; 16] {
        let uuid: Uuid = self.0;
        let packed: u16 = u16::from_be_bytes([uuid.data4[0], uuid.data4[1]]);
        let mut bytes: [u8; 16] = [0; 16];
        bytes[0..4].copy_from_slice(&uuid.data1.to_ne_bytes());
        bytes[4..6].copy_from_slice(&uuid.data2.to_ne_bytes());
        bytes[6..8].copy_from_slice(&uuid.data3.to_ne_bytes());
        bytes[8..10].copy_from_slice(&packed.to_ne_bytes());
        bytes[10..16].copy_from_slice(&uuid.data4[2..8]);
        bytes
    }

    /// `std::hash<Standard_GUID>::operator()`: `hashBytes` over the 16-byte field image
    /// (`int`, three `char16_t`, six `uint8_t`; no padding).
    ///
    /// ```
    /// use rustycad_tkernel::standard::Guid;
    /// assert_eq!(Guid::new().hash_code(), rustycad_tkernel::standard::hash_bytes(&[0; 16]));
    /// ```
    pub fn hash_code(self) -> usize {
        hash_bytes(&self.field_bytes())
    }
}

impl std::hash::Hash for Guid {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_usize(self.hash_code());
    }
}

impl From<Uuid> for Guid {
    fn from(uuid: Uuid) -> Self {
        Self::from_uuid(uuid)
    }
}

impl From<Guid> for Uuid {
    fn from(guid: Guid) -> Self {
        guid.to_uuid()
    }
}
