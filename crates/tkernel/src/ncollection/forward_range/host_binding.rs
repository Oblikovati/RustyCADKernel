use std::cell::RefCell;

// OCCT's const range owns a mutable cursor. Short-lived cell borrows let two
// adapters observe that cursor without manufacturing aliased mutable references.
pub(super) enum HostBinding<'a, I> {
    Exclusive(&'a mut I),
    Shared(&'a RefCell<I>),
}

impl<I> HostBinding<'_, I> {
    pub(super) fn read<R>(&self, inspect: impl FnOnce(&I) -> R) -> R {
        match self {
            Self::Exclusive(host) => inspect(host),
            Self::Shared(host) => inspect(&host.borrow()),
        }
    }

    pub(super) fn change<R>(&mut self, update: impl FnOnce(&mut I) -> R) -> R {
        match self {
            Self::Exclusive(host) => update(host),
            Self::Shared(host) => update(&mut host.borrow_mut()),
        }
    }
}
