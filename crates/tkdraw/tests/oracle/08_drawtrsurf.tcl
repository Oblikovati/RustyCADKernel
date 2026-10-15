# DrawTrSurf drawables: restore from hand-written records, dtyp, dump, copy, bsave, and
# the display and transformation commands (no window: nothing is drawn).
proc write_record {file lines} {
  set channel [open $file w]
  foreach line $lines { puts $channel $line }
  close $channel
}
proc show_saved {name} {
  uplevel #0 [list bsave $name tkdraw_oracle_saved.draw]
  set channel [open tkdraw_oracle_saved.draw r]
  set text [string map {\n |} [read $channel]]
  close $channel
  file delete tkdraw_oracle_saved.draw
  return $text
}
proc restore_record {name lines} {
  write_record tkdraw_oracle_record.draw $lines
  # Draw variables are Tcl variables of the calling scope: restore at global level.
  set result [uplevel #0 [list brestore tkdraw_oracle_record.draw $name]]
  file delete tkdraw_oracle_record.draw
  return $result
}

puts "curve: [restore_record c {DrawTrSurf_Curve {2 0 0 0 0 0 1 1 0 0 0 1 0 5}}] <[dtyp c]>"
puts [dump c]
puts "saved: [show_saved c]"
puts "bezier: [restore_record bz {DrawTrSurf_BezierCurve {6 0 2 0 0 0 1 1 0 2 0 0}}] <[dtyp bz]>"
puts [dump bz]
puts "saved: [show_saved bz]"
puts "bspline: [restore_record bs {DrawTrSurf_BSplineCurve {7 0 0 2 4 3 0 0 0 1 2 0 2 2 0 3 0 0 0 3 1 1 2 3}}] <[dtyp bs]>"
puts [dump bs]
puts "saved: [show_saved bs]"
puts "rational bspline: [restore_record rbs {DrawTrSurf_BSplineCurve {7 1 0 2 3 2 0 0 0 1 1 0 2 1 2 0 0 1 0 3 1 3}}]"
puts [dump rbs]
puts "line2d: [restore_record l2 {DrawTrSurf_Curve2d {1 1 2 0.6 0.8}}] <[dtyp l2]>"
puts [dump l2]
puts "saved: [show_saved l2]"
puts "bezier2d: [restore_record b2 {DrawTrSurf_BezierCurve2d {6 0 2 0 0 1 1 2 0}}] <[dtyp b2]>"
puts [dump b2]
puts "bspline2d: [restore_record s2 {DrawTrSurf_BSplineCurve2d {7 0 0 1 2 2 0 0 3 4 0 2 1 2}}] <[dtyp s2]>"
puts [dump s2]
puts "saved: [show_saved s2]"
puts "plane: [restore_record p {DrawTrSurf_Surface {1 0 0 0 0 0 1 1 0 0 0 1 0}}] <[dtyp p]>"
puts [dump p]
puts "saved: [show_saved p]"
puts "bezier surface: [restore_record bzs {DrawTrSurf_BezierSurface {8 0 0 1 1 0 0 0 0 1 0 1 0 0 1 1 1}}] <[dtyp bzs]>"
puts [dump bzs]
puts "bspline surface: [restore_record bss {DrawTrSurf_BSplineSurface {9 0 0 0 0 1 1 2 2 2 2 0 0 0 0 1 0 1 0 0 1 1 1 0 2 1 2 0 2 1 2}}] <[dtyp bss]>"
puts [dump bss]
puts "saved: [show_saved bss]"
puts "point: [restore_record pt {DrawTrSurf_Point {1 1.5 -2 3}}] <[dtyp pt]>"
puts [dump pt]
puts "saved: [show_saved pt]"
puts "point2d: [restore_record p2 {DrawTrSurf_Point {0 0.25 4}}] <[dtyp p2]>"
puts [dump p2]
puts "saved: [show_saved p2]"

puts "copy: [copy bs bs2] <[dtyp bs2]> [copy bss bss2] <[dtyp bss2]> [copy pt pt2] <[dtyp pt2]>"
puts "defaults: [dmode] [discr] [defle] [nbiso]"
puts "set defaults: [discr 12] [defle 0.5] [nbiso 2 3] [dmode u]"
puts "named: [dmode bs u] [discr bs c 40] [defle bs c 0.1] [nbiso p bss 4 5]"
puts "poles: [shpoles bs] [clpoles bz] [catch {clknots bz}] [clknots bs] [shknots bss] [catch {clpoles c}] [clpoles]"
puts "colors: [setcurvcolor] [setcurvcolor GOLDEN] [setcurvcolor] [setcurvcolor nonsense] [setcurvcolor]"
puts "change: [changecurvcolor red bs] [catch {changecurvcolor red}] [changepointcolor blue pt] [changepointmarker circle pt]"
puts "markers: [setpointmarker] [setpointmarker circle_zoom] [setpointmarker] [setpointcolor darkgreen] [setpointcolor]"

puts "translate: [translate c pt 1 2 3]"
puts [dump c pt]
puts "rotate: [rotate bs 0 0 0 0 0 1 90]"
puts [dump bs]
puts "lmirror: [lmirror bz 0 0 0 1 0 0]"
puts [dump bz]
puts "smirror: [smirror p 0 0 1 0 0 1]"
puts [dump p]
puts "pmirror: [pmirror pt 0 0 0]"
puts [dump pt]
puts "pscale: [pscale bss 1 1 1 2]"
puts [dump bss]
puts "2dtranslate: [2dtranslate l2 p2 1 1]"
puts [dump l2 p2]
puts "2drotate: [2drotate s2 0 0 90]"
puts [dump s2]
puts "2dlmirror: [2dlmirror b2 0 0 1 1]"
puts [dump b2]
puts "2dpmirror: [2dpmirror p2 1 1]"
puts [dump p2]
puts "2dpscale: [2dpscale s2 0 0 3]"
puts [dump s2]
puts "short: [catch {translate c 1 2} message] <$message> [catch {2drotate c 1} message] <$message>"
puts "rotate null axis: [catch {rotate c 0 0 0 0 0 0 10} message] <[string range $message 0 30]>"

# Polygons and triangulations: Dump inherits the dump stream precision and Save sets the
# stream to scientific notation with 15 digits before Poly::Write.
puts "polygon3d: [restore_record q3 {DrawTrSurf_Polygon3D Poly_Polygon3D {3 1} 0.125 {0 0.5 1} {2.25 3 -4} {0.1 0.2 0.3} {0 0.5 1.75}}] <[dtyp q3]>"
puts [dump q3]
puts [show_saved q3]
puts "polygon2d: [restore_record q2 {DrawTrSurf_Polygon2D Poly_Polygon2D {2 0.5} {0 1.5} {2.75 -3}}] <[dtyp q2]>"
puts [dump q2]
puts [show_saved q2]
puts "triangulation: [restore_record tr {DrawTrSurf_Triangulation Poly_Triangulation {3 1 0} 0.25 {0 0 0} {1 0 0} {0 0.1 0} {1 2 3}}] <[dtyp tr]>"
puts [dump tr]
puts [show_saved tr]
