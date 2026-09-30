use std::marker::PhantomData;
/// Implements one actual host accessor; `ReadAccessor<Host>` links its output to a Rust type.
pub trait ReadAccessor<I> {
    /// Reference or value produced by this accessor; e.g. `&'a Point`.
    type Item;
    /// Unqualified snapshot type; e.g. `Point`.
    type Value;
    /// Calls the host accessor; `A::read(&host)`.
    fn read(host: &I) -> Self::Item;
    /// Copies a reference result or moves a value result; `A::snapshot(item)`.
    fn snapshot(item: Self::Item) -> Self::Value;
}
/// Declares an absent method; `AccessorSet<Present<A>,Missing,Missing>`.
pub struct Missing;
/// Declares an implemented accessor; `Present<ValueAccessor>`.
pub struct Present<A>(PhantomData<A>);
/// Compile-checked accessor availability; `Present<A>` requires `A:ReadAccessor<I>`.
pub trait AccessorPresence<I> {
    /// Whether the associated host method exists; `Present<A>::PRESENT`.
    const PRESENT: bool;
}
impl<I> AccessorPresence<I> for Missing {
    const PRESENT: bool = false;
}
impl<I, A: ReadAccessor<I>> AccessorPresence<I> for Present<A> {
    const PRESENT: bool = true;
}
/// Accessors in Value, Current, CurrentId priority order; `AccessorSet<Missing,Present<A>,Missing>`.
pub struct AccessorSet<V, C, D>(PhantomData<(V, C, D)>);
/// Availability metadata for the declared accessor set; `HAS_VALUE`.
pub trait AccessorMetadata {
    /// Reports whether Value is declared; `Set::HAS_VALUE`.
    const HAS_VALUE: bool;
    /// Reports whether Current is declared; `Set::HAS_CURRENT`.
    const HAS_CURRENT: bool;
    /// Reports whether CurrentId is declared; `Set::HAS_CURRENT_ID`.
    const HAS_CURRENT_ID: bool;
}
/// Selects the highest-priority implemented method; `Set::read(&host)`.
pub trait SelectAccessor<I>: AccessorMetadata {
    /// The selected reference or value type; `Set::Item`.
    type Item;
    /// The unqualified snapshot type; `Set::Value`.
    type Value;
    /// Reads from the selected method; `Set::read(&host)`.
    fn read(host: &I) -> Self::Item;
    /// Captures a postfix snapshot; `Set::snapshot(item)`.
    fn snapshot(item: Self::Item) -> Self::Value;
}
// Separate shape metadata from the host-specific callable proof, so absent methods
// cannot accidentally win dispatch and lower-priority declarations remain checked.
trait Declared {
    const PRESENT: bool;
}
impl Declared for Missing {
    const PRESENT: bool = false;
}
impl<A> Declared for Present<A> {
    const PRESENT: bool = true;
}
impl<V: Declared, C: Declared, D: Declared> AccessorMetadata for AccessorSet<V, C, D> {
    const HAS_VALUE: bool = V::PRESENT;
    const HAS_CURRENT: bool = C::PRESENT;
    const HAS_CURRENT_ID: bool = D::PRESENT;
}
impl<I, A: ReadAccessor<I>, C: AccessorPresence<I> + Declared, D: AccessorPresence<I> + Declared>
    SelectAccessor<I> for AccessorSet<Present<A>, C, D>
{
    type Item = A::Item;
    type Value = A::Value;
    fn read(host: &I) -> Self::Item {
        A::read(host)
    }
    fn snapshot(item: Self::Item) -> Self::Value {
        A::snapshot(item)
    }
}
impl<I, A: ReadAccessor<I>, D: AccessorPresence<I> + Declared> SelectAccessor<I>
    for AccessorSet<Missing, Present<A>, D>
{
    type Item = A::Item;
    type Value = A::Value;
    fn read(host: &I) -> Self::Item {
        A::read(host)
    }
    fn snapshot(item: Self::Item) -> Self::Value {
        A::snapshot(item)
    }
}
impl<I, A: ReadAccessor<I>> SelectAccessor<I> for AccessorSet<Missing, Missing, Present<A>> {
    type Item = A::Item;
    type Value = A::Value;
    fn read(host: &I) -> Self::Item {
        A::read(host)
    }
    fn snapshot(item: Self::Item) -> Self::Value {
        A::snapshot(item)
    }
}
