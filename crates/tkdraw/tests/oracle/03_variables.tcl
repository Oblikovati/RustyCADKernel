# Draw variables: numbers, protection, copy/rename, type queries and traces.
puts "standard: [isdraw axes] [isprot axes] [isdraw pi] [isprot pi] [isdraw grid] [isdraw nothing]"
puts "types: [dtyp pi] [dtyp axes] [dtyp axes2d] [dtyp grid]"
puts "pi=[dval pi] pinf=[dval pinf] minf=[dval minf]"
dset n 3
puts "n: [isdraw n] [isprot n] [dtyp n] [dname n] value [set n]"
puts "protect: [protect n] [isprot n]"
puts "set protected: [catch {set n 5} message] <$message> value [set n] dval [dval n]"
puts "dset protected: [dset n 7] -> [dval n]"
puts "unprotect: [unprotect n] [isprot n]"
set n 9
puts "after set: [isdraw n] <[set n]>"
dset a 1.5
copy a b
puts "copy: [dval b] [isdraw a] [isdraw b]"
dset b 2
puts "independent: [dval a] [dval b]"
renamevar a c
puts "renamevar: [isdraw a] [isdraw c] [dval c] [dname c]"
renamevar c d c2 d2
puts "renamevar pairs: [isdraw d] [isdraw c2] [isdraw d2]"
copy pi mypi
puts "copy of protected: [isprot mypi] [dval mypi]"
unset d
puts "unset: [isdraw d] [info exists d]"
puts "dump: [string map {\n |} [dump b]]"
puts "autodisplay: [autodisplay] [autodisplay] [autodisplay 0] [autodisplay 1]"
puts "display/erase: <[display b]> <[erase b]> <[donly b]> <[clear]> <[2dclear]> <[erase]>"
puts "repaint: <[repaint]>"
puts "draw: [catch {draw 1 3 b} message]"
puts "dgetenv: <[dgetenv TKDRAW_ORACLE_UNSET]>"
dsetenv TKDRAW_ORACLE_VAR hello
puts "dsetenv: <[dgetenv TKDRAW_ORACLE_VAR]>"
dsetenv TKDRAW_ORACLE_VAR
puts "dsetenv removed: <[dgetenv TKDRAW_ORACLE_VAR]>"
puts "lastrep: [lastrep id x y b] [dval id] [dval x] [dval y] [dval b]"
proc local {} { dset loc 1; return [isdraw loc] }
puts "local: [local] [isdraw loc]"
puts "array element: [catch {dset arr(1) 2} message] <$message>"
