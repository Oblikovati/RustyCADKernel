# Draw_Interpretor: results, errors, help and group registration, echo and log.
puts "batch: [batch]"
puts "dset result: [dset a 1 b 2.5]"
puts "help dset: $Draw_Helps(dset)"
foreach group {{DRAW General Commands} {DRAW Variables management} {DRAW Numeric functions}
               {DRAW Unit Commands} {DRAW Message Commands} {DRAW Graphic Commands} {Draw Plugin}} {
  puts "$group: [lsort $Draw_Groups($group)]"
}
puts "file: [info exists Draw_Files(dset)]"
puts "catch: [catch {dval} message] <$message>"
puts "unknown: [catch {nosuchcommand} message] <$message>"
puts "dlog status: [dlog status]"
dlog on
puts "dlog status: [dlog status]"
dlog add "added line"
dval 1+1
puts "log: [string map {\n |} [dlog get]]"
dlog reset
puts "log after reset: <[dlog get]>"
dlog off
puts "dlog bad: [catch {dlog bad} message] <$message>"
decho on
dval 2*3
decho off
puts "decho bad: [catch {decho maybe} message]"
puts "complete script result follows"
dval 6*7
