# Private nested template aliases in compiler inventory

This capture feature records public reachability through public aliases without changing the visibility of the original generic declaration rows.

## Event schema

When a visible `TypedefNameDecl` resolves to a specialization of a private nested `ClassTemplateDecl`, the extractor emits one `alias_reachability` event in addition to the unchanged declaration events. The event identifies the alias and target by USR and qualified name, retains each target template argument as `{index, parameter, kind, text, dependent}`, and records direct public/protected members from the template definition. Each member includes its original declaration record and a `visibility_basis` of `public_or_protected_member_of_alias_target`.

The member's nested declaration keeps its source visibility (`api_visible=false`, often `private_ancestor`). `api_visible_through_alias=true` is a separate relationship signal. `template_constraints_evaluated=false` and `member_constraints_evaluated=false` mean neither class nor member-template viability is inferred from the alias binding. `member_surface_basis="primary_template_pattern"` identifies the source for listed members; explicit-specialization member replacement is not resolved. The original member template signatures and exact alias specialization arguments remain present for a later verifier.

The merge pipeline retains these events in `occt-alias-reachability.jsonl.gz` and reports record/member counts separately from declaration counts. Every event carries `proof_status="supplemental_unresolved"` and `coverage_credit=false`: it does not count as a declaration row, alter `api_declarations`, or establish coverage proof. Unknown future event kinds still enter `unsupported`.

## Deliberate limits

- The supported target is a class-template specialization reached through a visible typedef/type alias. Public aliases of non-template private records, alias-template specializations, and other target forms are not resolved by this event producer. If the target remains incomplete, `target_definition_available=false` makes the missing member surface explicit.
- The event lists direct public/protected members only. It does not recursively add inherited members or resolve `using` declarations to their base declaration targets.
- Class/member-template constraints and overload viability are retained, not evaluated after substituting the alias's template arguments. Explicit class-template specialization member surfaces are not selected; listed declarations are from the primary pattern.
- This relationship evidence does not rewrite hidden target names to the public alias in callable signatures. Consumers must use the alias edge when generating usable public selectors.

## Validated fixtures

The extractor fixture checks `<true>` and `<false>` specializations, a chain alias, a protected alias, a private alias exclusion, public targets that must not produce private-target exposures, and both struct-default-public and class-default-private member access. The alias target has public, protected, and private members plus a constrained member template; only public/protected members are recorded, and the member template constraint remains explicit. Positive C++ code calls through public aliases and a derived type. Negative C++ code directly names the private nested template or calls a private member through the public alias; both must fail.

At the pinned OCCT source, the separate `NCollection_FlatMap.hxx` and `NCollection_FlatDataMap.hxx` probes each produced two alias events. They preserve respectively `iterator_impl<true>` and `iterator_impl<false>`/`iterator_impl<true>`. Both retain `Key`, `More`, dereference, and increment while omitting private helper methods and fields.
