# Martensite visible-field exclusion audit

This candidate audit joins existing private-state exclusions with compiler-visible public/protected declarations. It found 174 visualization candidates and 1,286 candidates across all toolkits. Candidates require individual source and behavioral review; this receipt does not change their status or claim their completion. Eight public ImageDumpOptions value fields already have mutable Rust counterparts and should be mapped accurately. Missing protected mutation and native alias lifetimes require actual porting work, rather than blanket exclusions.
