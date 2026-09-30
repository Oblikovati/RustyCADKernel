//! The OSD enumerations, with the native declaration order as `i32` discriminants.
//!
//! Each enum keeps the C++ order because OCCT code (and the protection tables) index
//! by the raw value; `from_raw` is the checked inverse of `as i32`.

macro_rules! osd_enum {
    ($(#[$meta:meta])* $name:ident { $($(#[$vmeta:meta])* $variant:ident),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        #[repr(i32)]
        pub enum $name {
            $($(#[$vmeta])* $variant),+
        }
        impl $name {
            /// Every value in native declaration order; `ALL[raw as usize]` inverts `as i32`.
            pub const ALL: &'static [Self] = &[$(Self::$variant),+];
            /// Converts a native discriminant, or `None` outside the enumeration;
            #[doc = concat!("`", stringify!($name), "::from_raw(0)`.")]
            pub fn from_raw(raw: i32) -> Option<Self> {
                usize::try_from(raw).ok().and_then(|index| Self::ALL.get(index).copied())
            }
        }
    };
}

osd_enum! {
    /// `OSD_SysType`: the operating-system dialect a path is spelled in.
    SysType {
        /// `OSD_Unknown`.
        Unknown,
        /// `OSD_Default`: the dialect of the running host.
        Default,
        /// `OSD_UnixBSD`.
        UnixBsd,
        /// `OSD_UnixSystemV`.
        UnixSystemV,
        /// `OSD_VMS`.
        Vms,
        /// `OSD_OS2`.
        Os2,
        /// `OSD_OSF`.
        Osf,
        /// `OSD_MacOs`.
        MacOs,
        /// `OSD_Taligent`.
        Taligent,
        /// `OSD_WindowsNT`.
        WindowsNt,
        /// `OSD_LinuxREDHAT`.
        LinuxRedHat,
        /// `OSD_Aix`.
        Aix,
    }
}

osd_enum! {
    /// `OSD_SingleProtection`: read (1), write (2), execute (4) and delete (8) bits.
    SingleProtection {
        /// `OSD_None`.
        None,
        /// `OSD_R`.
        R,
        /// `OSD_W`.
        W,
        /// `OSD_RW`.
        Rw,
        /// `OSD_X`.
        X,
        /// `OSD_RX`.
        Rx,
        /// `OSD_WX`.
        Wx,
        /// `OSD_RWX`.
        Rwx,
        /// `OSD_D`.
        D,
        /// `OSD_RD`.
        Rd,
        /// `OSD_WD`.
        Wd,
        /// `OSD_RWD`.
        Rwd,
        /// `OSD_XD`.
        Xd,
        /// `OSD_RXD`.
        Rxd,
        /// `OSD_WXD`.
        Wxd,
        /// `OSD_RWXD`.
        Rwxd,
    }
}

osd_enum! {
    /// `OSD_FromWhere`: the origin of a file seek.
    FromWhere {
        /// `OSD_FromBeginning`.
        FromBeginning,
        /// `OSD_FromHere`.
        FromHere,
        /// `OSD_FromEnd`.
        FromEnd,
    }
}

osd_enum! {
    /// `OSD_KindFile`: the kind of a file-system node.
    KindFile {
        /// `OSD_FILE`.
        File,
        /// `OSD_DIRECTORY`.
        Directory,
        /// `OSD_LINK`.
        Link,
        /// `OSD_SOCKET`.
        Socket,
        /// `OSD_UNKNOWN`.
        Unknown,
    }
}

osd_enum! {
    /// `OSD_LockType`: the lock held on an open file.
    LockType {
        /// `OSD_NoLock`.
        NoLock,
        /// `OSD_ReadLock`.
        ReadLock,
        /// `OSD_WriteLock`.
        WriteLock,
        /// `OSD_ExclusiveLock`.
        ExclusiveLock,
    }
}

osd_enum! {
    /// `OSD_OpenMode`: the access mode of an open file.
    OpenMode {
        /// `OSD_ReadOnly`.
        ReadOnly,
        /// `OSD_WriteOnly`.
        WriteOnly,
        /// `OSD_ReadWrite`.
        ReadWrite,
    }
}

osd_enum! {
    /// `OSD_OEMType`: the hardware vendor of a host.
    OemType {
        /// `OSD_Unavailable`.
        Unavailable,
        /// `OSD_SUN`.
        Sun,
        /// `OSD_DEC`.
        Dec,
        /// `OSD_SGI`.
        Sgi,
        /// `OSD_NEC`.
        Nec,
        /// `OSD_MAC`.
        Mac,
        /// `OSD_PC`.
        Pc,
        /// `OSD_HP`.
        Hp,
        /// `OSD_IBM`.
        Ibm,
        /// `OSD_VAX`.
        Vax,
        /// `OSD_LIN`.
        Lin,
        /// `OSD_AIX`.
        Aix,
    }
}

osd_enum! {
    /// `OSD_LoadMode`: when a shared library resolves its symbols.
    LoadMode {
        /// `OSD_RTLD_LAZY`.
        RtldLazy,
        /// `OSD_RTLD_NOW`.
        RtldNow,
    }
}

osd_enum! {
    /// `OSD_WhoAmI`: the OSD class that recorded an error, selecting its message.
    WhoAmI {
        /// `OSD_WDirectory`.
        Directory,
        /// `OSD_WDirectoryIterator`.
        DirectoryIterator,
        /// `OSD_WEnvironment`.
        Environment,
        /// `OSD_WFile`.
        File,
        /// `OSD_WFileNode`.
        FileNode,
        /// `OSD_WFileIterator`.
        FileIterator,
        /// `OSD_WPath`.
        Path,
        /// `OSD_WProcess`.
        Process,
        /// `OSD_WProtection`.
        Protection,
        /// `OSD_WHost`.
        Host,
        /// `OSD_WDisk`.
        Disk,
        /// `OSD_WChronometer`.
        Chronometer,
        /// `OSD_WTimer`.
        Timer,
        /// `OSD_WPackage`.
        Package,
        /// `OSD_WEnvironmentIterator`.
        EnvironmentIterator,
    }
}

osd_enum! {
    /// `OSD_SignalMode`: how `OSD::SetSignal` treats the process signal handlers.
    SignalMode {
        /// `OSD_SignalMode_AsIs`: do not set or remove signal handlers.
        AsIs,
        /// `OSD_SignalMode_Set`: set OCCT signal handlers.
        Set,
        /// `OSD_SignalMode_SetUnhandled`: set OCCT handlers only where none is set.
        SetUnhandled,
        /// `OSD_SignalMode_Unset`: restore the system default handlers.
        Unset,
    }
}
