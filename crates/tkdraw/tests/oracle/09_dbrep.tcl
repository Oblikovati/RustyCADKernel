# DBRep: shape variables read from a BRep file, the counting, exploding and modifying
# commands, the display settings and the BRep file round trips (no window).
set data "../../../tktopalgo/tests/data/brep_gprop/occ49-cylinder-minus-box.brep"
puts "brestore: [brestore $data s] <[dtyp s]>"
puts "readbrep: [readbrep $data r] <[dtyp r]>"
puts [nbshapes s]
puts [nbshapes s -t]
puts [numshapes s]
puts [countshapes s]
puts "explode: [explode s]"
puts "explode faces: [explode s f]"
puts "explode edges: [llength [explode s e]] [dtyp s_1]"
puts "explode bad: [catch {explode s x}]"
# The dump shows the TShape addresses: mask them.
puts "dump edge: [regsub -all {0x[0-9a-f]+} [dump s_1] {0x?}]"
explode s v
puts "dump vertex: [regsub -all {0x[0-9a-f]+} [dump s_1] {0x?}]"
puts "nexplode faces: [nexplode s f]"
puts "nexplode vertices: [nexplode s v]"
foreach v {s_1 s_2 s_3 s_4} { puts "$v: [dtyp $v]" }
explode s w
puts "exwire: [exwire s_1] [dtyp s_1_1] [dtyp s_1_2]"
puts "compound: [compound s_1 s_2 c] <[dtyp c]> [nbshapes c]"
puts "add: [add s_3 c] [nbshapes c]"
puts "emptycopy: [emptycopy e s] <[dtyp e]> [nbshapes e]"
puts "orientation: [orientation s_1 s_2 R] <[dtyp s_1]> <[dtyp s_2]>"
puts "treverse: [treverse s_1] <[dtyp s_1]>"
puts "complement: [complement s_1] <[dtyp s_1]>"
puts "orientation I: [orientation s_1 I] <[dtyp s_1]> [orientation s_1 E] <[dtyp s_1]>"
puts "invert: [invert c] <[dtyp c]>"
puts "setflags: [setflags c free -modified closed convex] <[dtyp c]>"
puts "setflags clear: [setflags c -free -closed -convex] <[dtyp c]>"
puts "isos: [isos] [isos 3] [isos] [isos s] [isos s 4] [isos s] [catch {isos s 1.5}]"
puts "discretisation: [discretisation] [discretisation 12] [discretisation]"
puts "hlr: [hlr]"
puts "hlr modes: [hlr hlr] [hlr]"
puts "hlr flags: [hlr rgn] [hlr] [hlr hid] [hlr] [hlr norg1] [hlr nohid] [hlr]"
puts "hlr ang: [hlr ang] [hlr ang 10] [hlr ang 90] [hlr ang 0.1] [hlr ang 20 s]"
puts "hlr shapes: [hlr nohlr s] [catch {hlr bogus}] [catch {hlr bogus s}] [hlr nohlr]"
puts "triangles: [triangles] [triangles s] [polygons] [polygons s]"
puts "vconn: [vconn s]"
puts "check: [check s] tclean: [tclean s] [tclean -geom c] [catch {tclean -geom -force s}] [catch {tclean nothing}]"
puts "removeinternals: [removeinternals s] [removeinternals s 1] [catch {removeinternals}]"
puts "XProgress: [XProgress] [XProgress +t] [XProgress -t -c +g] [XProgress -t -g]"
puts "writebrep: [writebrep s tkdraw_oracle_ascii.brep] [readbrep tkdraw_oracle_ascii.brep a] [nbshapes a]"
puts "binsave: [binsave s tkdraw_oracle_binary.bin] [binrestore tkdraw_oracle_binary.bin b] [nbshapes b]"
puts "writebrep options: [writebrep s tkdraw_oracle_v1.brep -version 1 -triangles 0] [readbrep tkdraw_oracle_v1.brep v1] [dtyp v1]"
puts "writebrep binary: [writebrep s tkdraw_oracle_v2.bin -binary -version 2] [readbrep tkdraw_oracle_v2.bin v2] [dtyp v2]"
puts "writebrep errors: [catch {writebrep s}] [catch {writebrep nothing f}] [catch {writebrep s f -version 9}] [catch {writebrep s f -binary -version 9}]"
puts "readbrep errors: [catch {readbrep}] [catch {readbrep no_such_file x}]"
puts "bsave: [bsave s tkdraw_oracle_shape.draw] [brestore tkdraw_oracle_shape.draw t] [dtyp t] [nbshapes t]"
file delete tkdraw_oracle_ascii.brep tkdraw_oracle_binary.bin tkdraw_oracle_v1.brep tkdraw_oracle_v2.bin tkdraw_oracle_shape.draw
# The C++ map of faces iterates in hash (address) order: compare the sorted lines.
puts "normals: [join [lsort [split [normals s 2 -print] \n]] |]"
puts "normals options: [join [lsort [split [normals s -length 3 -nbu 2 -nbv 2 -print] \n]] |]"
puts "normals errors: [catch {normals}] [catch {normals nothing}] [catch {normals s 0}] [catch {normals s -nbu 0}] [catch {normals s -bogus}]"
