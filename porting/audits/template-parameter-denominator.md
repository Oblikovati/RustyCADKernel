# Template parameter inventory classification

## Finding

The published compiler crosswalk counts template parameter AST nodes as
independent API candidates. The captured snapshot contains 1,777 such
unclaimed rows: 1,502 `TemplateTypeParm`, 274 `NonTypeTemplateParm`, and 1
`TemplateTemplateParm`, out of 83,371 unclaimed declarations. The report filters
only `api_visible` in `tools/api_coverage_report.py:95-96,265-273`, so these
rows enlarge the denominator even though Rust has no corresponding callable or
nominal item for each C++ template parameter.

The source example is pinned OCCT revision
`3d097a0328e71b826377d4814ab05ec3c3d23871`,
`src/FoundationClasses/TKernel/Standard/Standard_Handle.hxx:52-53`, where
`T` parameterizes the `opencascade::handle` class template. The API surface is
the template and its operations; `T` supplies a type argument to that surface.
The extracted row has `kind=TemplateTypeParm`, qualified name `T`, and no
callable signature or standalone canonical type. Its parent template
declaration is the correct inventory obligation.

## Change

`tools/api_inventory/src/Schema.cpp` now retains parameter events but marks
`TemplateTypeParmDecl`, `NonTypeTemplateParmDecl`, and
`TemplateTemplateParmDecl` as `api_visible=false` with
`exclusion_reason=template_parameter_metadata`. Template parameter lists are
attached to `ClassTemplateDecl`, `FunctionTemplateDecl`, `TypeAliasTemplateDecl`,
`VarTemplateDecl`, their patterns, and partial specializations as normalized
`template_signature` metadata. This keeps bounds, default arguments, packs,
and nested template-template parameter lists observable without creating a
separate Rust API claim for each parameter.

The fixture at
`tools/api_inventory/fixtures/source/include/Fixture.hxx` covers a class
template with a defaulted non-type parameter, an associated-type substitution
constraint, a parameter pack, a template-template parameter, a type-alias
template, and a function template.
It verifies that wrapper declarations remain candidates, template parameters
are retained only as exclusions, private nested declarations remain excluded,
and ordinary aliases remain candidates. The test helper checks preserved
parameter-list metadata rather than relying on names as API paths.

Source hashes for this change are:

- Pinned OCCT handle header: `17324edfa4eac229897635d359554f699e70e1c95522c17dadf7c8c34644c25f`
- Extractor: `08c99a48a402ad492867139d970c4aa7ca509d4623fec73b17afe4dc74670546`
- Compiler fixture: `48605ffea2e2b490250dc949fd4b9eefc672e55da1740381907f07a35bc9f33b`
- Fixture assertions: `9f2f4b2ea2bf17fead8a4ac28a8018df0a25b455df451706714c864ab8d839e7`

## Validation and snapshot status

Built with pinned Clang 23.1.2 from the repository-owned API proof toolchain.
`ctest --test-dir /tmp/rustycad-template-parameter-denominator-build
--output-on-failure` passes both extractor fixture and signature-identity
tests.

The committed crosswalk used for the 1,777-row count has SHA-256
`6e1a2d0b2709b8eb9347fc052bbc7b57e4bca54156095602122213f2f4580b6c`. Its
counts and generated artifacts have not been edited or relabeled. A fresh
compiler capture and normalization is required before publishing the reduced
candidate denominator and its new provenance.


## Fresh capture at Rust revision e8f0f360

The rebuilt extractor was run over all 5,905 configured standalone header probes,
then 266 failed probes were retried in real source contexts. The validated
transitive-context plan supplied 16 additional successful contexts. The combined
result retains 5,639 standalone parses, 260 context parses, six parse failures,
384 included fragments, and 28 missing configurations.

All **271,643 declaration identities** remain in the published compiler evidence.
An identity-by-identity comparison confirms that the only visibility changes are
1,502 type parameters, 274 non-type parameters, and one template-template parameter,
now explicitly classified as `template_parameter_metadata`. No callable, nominal
class, template wrapper, or alias identity was removed. The observed API denominator
is **108,357**, down from 110,134 solely through this classification correction.
This changes inventory accounting, not the number of implemented Rust APIs.

The published `occt-manifest.json` binds the new extractor, source/header manifest,
compile database, SDK headers, and compressed artifacts by hash. Header gaps and
5,224 unsupported records remain explicit; all full-coverage proof flags remain false.
The Rust crosswalk was regenerated from source revision `e8f0f360`, with each of the
new Storage setters and PLib derivative hooks resolving to a unique native and Rust
item. Supplemental tests do not increase the 7,400/8,648 legacy declaration mapping count.

Capture intermediates are in `target/api-proof/occt-template-metadata*`.
`tools/capture_occt_inventory.py`, `tools/retry_occt_header_context.py`, and
`tools/replay_occt_header_context.py` generated the native evidence;
`tools/merge_occt_inventory.py` merged it. The combined context manifest retains
both failed original attempts and successful replay records, with input manifest hashes.
