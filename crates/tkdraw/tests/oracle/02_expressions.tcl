# Draw::Atof: the expression language of numeric arguments (dval prints %.17g).
dset x 4 y -2.5
foreach expression {
  1+2*3 8/2/2 2-3-4 -(1+2)*2 --3 +5 1.5e2 2e-1 12abc 0.1 1/3 2e 1.5e+ .5 5.
  x*2 x+y x/y -x (x) ((x+1)*(y-1)) x*x*x 1e308*10 -1e308*10 0/0
  cos(0) sin(0)+1 sqrt(16) atan2(1,1) cos(sqrt(0)) acos(1) tan(0)
  "1 + 2" " x " pi pinf minf
} {
  puts "dval $expression = [dval $expression]"
}
foreach expression {y+1 unknown 1(2 (1+2 2**3 1&2 f(1 x( 1 2} {
  puts "dval {$expression} = [catch {dval $expression} result] $result"
}
proc double {v} {return [expr {2*$v}]}
puts "user function: [dval double(21)]"
proc failing {v} {error "no"}
puts "failing function: [dval failing(1)+1]"
puts "nested: [dval double(double(2))]"
puts "result kept: [set r [dval 1]; dval double(3)]"
