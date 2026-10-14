# Private progress bound and borrowed member views

After native refusal correction `219824939`, the new private progress trait now
requires Send, and a compile-time test checks TwoLoopWork and the progress trait
object. This is not a claim that the synchronous borrowed native-owner graph or
local refusal is transferable. Successful immutable certificates retain their
separate Send + Sync assertions. No unsafe marker is used.

The member key, role and parent reexports mirror the annular binding worker's
visibility-only `f8064f619`, providing borrowed existing types without any new
authority constructor. No proof operand or serialization changed. Rust 1.99/jobs2
focused tests: 49 passed; complete V2 bytes equal the existing 43,373-byte pin.
The complete frozen-source matrix follows in final-commands.json.
