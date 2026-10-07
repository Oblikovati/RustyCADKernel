# Draw_BasicCommands without timing-dependent output.
dputs "plain text"
dputs -nonewline "no newline|"
dputs stdcout "after"
dputs -red -intense "colored when colors are on"
puts "dputs syntax: [catch {dputs -bogus} message]"
puts "dputs empty: [catch {dputs} message]"
puts "ddebugtraces: [ddebugtraces]"
ddebugtraces 3
puts "ddebugtraces set: [ddebugtraces]"
ddebugtraces 0
puts "ddebugtraces args: [catch {ddebugtraces 1 2} message] <$message>"
puts "dtracelevel: [dtracelevel]"
dtracelevel warning
puts "dtracelevel warning: [dtracelevel]"
dtracelevel info
puts "dtracelevel bad: [catch {dtracelevel loud} message]"
puts "dbreak: [dbreak]"
puts "dlocale numeric: [dlocale LC_NUMERIC]"
puts "dlocale bad: [catch {dlocale LC_BAD} message]"
puts "chrono flag: [chrono 1][chrono 0][chrono]"
chrono 0
chrono t -reset
puts "chrono var: [isdraw t] [dtyp t]"
puts "chrono bad: [catch {chrono t -bogus} message] <$message>"
puts "wait: [wait 0]"
puts "spy: [spy] [catch {spy /nonexistent/dir/file} message] <$message>"
puts "dsetsignal: [string map {\n |} [dsetsignal set 0]]"
puts "dsetsignal bad: [catch {dsetsignal sometimes} message]"
puts "meminfo bad: [catch {meminfo nothing} message] <$message>"
puts "dparallel bad: [catch {dparallel -nothing} message]"
puts "dtryload: [dtryload /nonexistent/libnothing.so]"
