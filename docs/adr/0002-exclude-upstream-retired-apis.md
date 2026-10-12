# Exclude OCCT deprecated and obsolete APIs from new Martensite ports

On 2026-10-12, the user removed APIs marked deprecated or obsolete by OCCT
from the required porting scope. Keep existing Rust implementations for now;
this decision does not authorize removing or breaking them.

Bind each exclusion to the pinned upstream declaration and its explicit marker:
compiler deprecation attributes, documentation marking that API obsolete or
deprecated, or an explicitly marked obsolete alias group. A retired class also
retires its member surface. An obsolete parameter, stale cache value, or mention
of an obsolete implementation technique does not retire an otherwise active API.

Keep the declaration, source evidence and exclusion reason in the inventory.
Existing implementation evidence remains available. Do not infer retirement
from age, naming, scheduling priority, or incomplete behavior. Visualization and
Vulkan remain the first priorities among active APIs.

The initial evidence-backed inventory update is recorded in
[the retirement audit](../../porting/audits/upstream-retirement-2026-10-12/README.md).
Future porting work must check explicit upstream retirement markers before
selecting remaining APIs. Unmatched identities remain open until proved.
