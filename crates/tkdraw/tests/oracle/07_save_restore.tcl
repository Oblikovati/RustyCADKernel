# bsave / brestore of numeric variables (Draw_Number Save/Restore).
dset s -2.5e-3
set file "tkdraw_oracle_number.draw"
puts "bsave: [bsave s $file]"
set channel [open $file r]
puts "file: [string map {\n |} [read $channel]]"
close $channel
puts "brestore: [brestore $file r] [dval r] [dtyp r]"
file delete $file
puts "bsave args: [catch {bsave s} message] <[string range $message 0 40]>"
puts "bsave not drawable: [catch {bsave nothing x} message] <$message>"
puts "bsave chronometer: [chrono c -reset; catch {bsave c $file} message] <$message>"
file delete -force $file
puts "brestore missing: [catch {brestore no_such_file.draw x} message] <$message>"
# A file whose type has no factory is restored as a DBRep_DrawableShape (BRep format),
# which needs DBRep; that case is not part of this oracle.
