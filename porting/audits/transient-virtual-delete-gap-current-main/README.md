# Martensite virtual Delete contract gap

Pinned `Standard_Transient::Delete() const` is a public virtual method, distinct
from the object's destructor. The current main ledger maps it to Rust
`Handle::drop` and treats a Rust destructor as the virtual override. That target
cannot implement a valid native override which retains the object at count zero.
The exact compiler ID, header position/hash and original row are in
`audit-row.json`; its requested status is partial.

The initial actual native probe publishes `1 0 0 0 2`, then `1`: replacing a
chain tail calls its overriding Delete once at count zero, without destroying
the object. Explicit subsequent `delete` destroys the still-constructed object
once. This uses a genuine derived plane, the normal owning chain field and no
extra keeper handle. The original temporary generator is preserved verbatim.

The focused durable generator has now run twice with identical raw output in
`native-1.txt` and `native-2.txt`. The receipt binds its source, binary, SDK
headers and libraries and separates those logs from the initial tool result;
no retained initial raw log is claimed. The extracted Rust fixture is
`crates/tkernel/tests/fixtures/transient-delete-retained-native.txt`.

The ordinary default destruction path must remain synchronous. A protocol fix
must distinguish virtual Delete dispatch from destruction, permit a real
retaining override and explicit later destruction, and avoid borrowing the
receiver across callbacks that can destroy it. A physical invocation lease must
not become a native owner or defer the default destructor. Existing scoped
retirement replay and copied conveniences do not close this contract.

Independent backing for the single actual reference counter is currently an
experimental branch checkpoint. It passes its focused strict Miri checks, but changes
allocation/layout costs. It has no integration or full parity credit before
identical accepted workload time, allocation and peak-memory measurements.
