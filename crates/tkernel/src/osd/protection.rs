//! `OSD_Protection`: system, user, group and world access rights of a file-system node.
use super::SingleProtection;

mod tables;

/// `OSD_Protection`; `Protection::new().user() == SingleProtection::Rwd` on Unix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Protection {
    system: SingleProtection,
    user: SingleProtection,
    group: SingleProtection,
    world: SingleProtection,
}
impl Default for Protection {
    fn default() -> Self {
        Self::new()
    }
}
impl Protection {
    /// The host default: Unix gives read to system, group and world and
    /// read/write/delete to the user; Windows gives RWXD to system and user and RX to
    /// group and world; `Protection::new()`.
    pub const fn new() -> Self {
        use SingleProtection::{R, Rwd, Rwxd, Rx};
        if cfg!(windows) {
            Self::with_values(Rwxd, Rwxd, Rx, Rx)
        } else {
            Self::with_values(R, Rwd, R, R)
        }
    }
    /// Stores explicit rights; `Protection::with_values(R, Rw, R, None)`.
    pub const fn with_values(
        system: SingleProtection,
        user: SingleProtection,
        group: SingleProtection,
        world: SingleProtection,
    ) -> Self {
        Self {
            system,
            user,
            group,
            world,
        }
    }
    /// Returns (system, user, group, world); `let (s, u, g, w) = protection.values()`.
    pub const fn values(
        &self,
    ) -> (
        SingleProtection,
        SingleProtection,
        SingleProtection,
        SingleProtection,
    ) {
        (self.system, self.user, self.group, self.world)
    }
    /// Replaces all four rights; `protection.set_values(R, Rw, R, None)`.
    pub const fn set_values(
        &mut self,
        system: SingleProtection,
        user: SingleProtection,
        group: SingleProtection,
        world: SingleProtection,
    ) {
        *self = Self::with_values(system, user, group, world);
    }
    /// Replaces the system right; `protection.set_system(SingleProtection::R)`.
    pub const fn set_system(&mut self, right: SingleProtection) {
        self.system = right;
    }
    /// Replaces the user right; `protection.set_user(SingleProtection::Rw)`.
    pub const fn set_user(&mut self, right: SingleProtection) {
        self.user = right;
    }
    /// Replaces the group right; `protection.set_group(SingleProtection::R)`.
    pub const fn set_group(&mut self, right: SingleProtection) {
        self.group = right;
    }
    /// Replaces the world right; `protection.set_world(SingleProtection::None)`.
    pub const fn set_world(&mut self, right: SingleProtection) {
        self.world = right;
    }
    /// Returns the system right; `protection.system()`.
    pub const fn system(&self) -> SingleProtection {
        self.system
    }
    /// Returns the user right; `protection.user()`.
    pub const fn user(&self) -> SingleProtection {
        self.user
    }
    /// Returns the group right; `protection.group()`.
    pub const fn group(&self) -> SingleProtection {
        self.group
    }
    /// Returns the world right; `protection.world()`.
    pub const fn world(&self) -> SingleProtection {
        self.world
    }
    /// `OSD_Protection::Add`: grants `right` in place; `protection.add(&mut rights, X)`.
    ///
    /// A member function that never reads the object, as natively.
    ///
    /// Unix reads the native 16x16 table, which differs from a bitwise union in two
    /// cells (WX+W gives RWX; RWX+D gives RWX); Windows is the exact union.
    pub fn add(&self, protection: &mut SingleProtection, right: SingleProtection) {
        *protection = if cfg!(windows) {
            tables::windows_add(*protection, right)
        } else {
            tables::unix_add(*protection, right)
        };
    }
    /// `OSD_Protection::Sub`: revokes `right` in place; `protection.sub(&mut rights, W)`.
    ///
    /// Unix reads the native table, which differs from a bitwise difference in three
    /// cells of the X row (X-RW and X-RWD give None; X-RWX gives X); Windows is exact.
    pub fn sub(&self, protection: &mut SingleProtection, right: SingleProtection) {
        *protection = if cfg!(windows) {
            tables::windows_sub(*protection, right)
        } else {
            tables::unix_sub(*protection, right)
        };
    }
    /// `OSD_Protection::Internal` (private, used by the file classes): the Unix mode bits
    /// `chmod`/`open` receive; `protection.internal() == 0o644` for the Unix default.
    ///
    /// Delete maps to write, and the system right is folded into the world bits. Windows
    /// returns 0.
    pub(crate) const fn internal(&self) -> u32 {
        if cfg!(windows) {
            return 0;
        }
        tables::mode_bits(self.user, 6)
            | tables::mode_bits(self.group, 3)
            | tables::mode_bits(self.world, 0)
            | tables::mode_bits(self.system, 0)
    }
}
