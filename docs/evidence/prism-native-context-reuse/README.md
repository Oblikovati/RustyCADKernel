# Native context correspondence reuse

The existing history copier, unchanged partition and generated directed-use matching retain their exact predicates and typed causes. Their lookup operands now borrow actual context slices, so the semantic resolver can reuse them before authored history exists. The former history wrappers still attach the same owned keys and causes. Bounded context readers expose the existing capture path.

All 45 existing history tests passed before and after. The 421,702-byte complete history replay is identical, including the copier-only intermediate. Strict all-target Clippy and formatting passed. The archive contains complete command logs and exact bytes. No correspondence is inferred from returned index or order, and no production map issuer is added here.
