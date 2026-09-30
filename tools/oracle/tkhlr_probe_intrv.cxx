// Intrv, the Rust twin of crates/tkhlr/examples/hl_probe/intrv.rs: fuzzy intervals, their
// relative positions and the boolean operations of interval sets.
#include <Intrv_Interval.hxx>
#include <Intrv_Intervals.hxx>
// `mode s ts e te`: 0 the default interval, 1 Intrv_Interval(s, e), 2 the constructor
// with tolerances (raised to Epsilon), 3 the default interval with SetStart and SetEnd
// (the tolerances are kept as given, even 0 or negative). Tolerances are narrowed to
// float as a caller's (float) cast does.
Intrv_Interval hl_iv_read(GaReader& r){
 int mode=r.integer();double s=r.real();float ts=(float)r.real();double e=r.real();float te=(float)r.real();
 if(mode==1)return Intrv_Interval(s,e);
 if(mode==2)return Intrv_Interval(s,ts,e,te);
 Intrv_Interval interval;if(mode==3){interval.SetStart(s,ts);interval.SetEnd(e,te);}
 return interval;}
void hl_iv_print(const Intrv_Interval& interval){
 double s,e;float ts,te;interval.Bounds(s,ts,e,te);
 ga_real(s);ga_real(ts);ga_real(e);ga_real(te);
 ga_real(interval.Start());ga_real(interval.TolStart());ga_real(interval.End());ga_real(interval.TolEnd());}
// `n interval...`: the set of the first interval united with the others in turn (empty
// for n = 0).
Intrv_Intervals hl_iv_read_set(GaReader& r){
 int n=r.integer();Intrv_Intervals set;
 for(int i=0;i<n;++i){Intrv_Interval interval=hl_iv_read(r);if(i==0)set=Intrv_Intervals(interval);else set.Unite(interval);}
 return set;}
void hl_iv_print_set(const Intrv_Intervals& set){
 ga_int(set.NbIntervals());for(int i=1;i<=set.NbIntervals();++i)hl_iv_print(set.Value(i));}
// `interval`: the bounds as read back, IsProbablyEmpty.
void hl_iv_interval_case(std::istream& input){
 GaReader r{input};Intrv_Interval interval=hl_iv_read(r);
 hl_iv_print(interval);ga_flag(interval.IsProbablyEmpty());ga_end();}
// `interval op value tol`: op 0 SetStart, 1 FuseAtStart, 2 CutAtStart, 3 SetEnd,
// 4 FuseAtEnd, 5 CutAtEnd; then the bounds.
void hl_iv_edit_case(std::istream& input){
 GaReader r{input};Intrv_Interval interval=hl_iv_read(r);int op=r.integer();double v=r.real();float t=(float)r.real();
 switch(op){
  case 0:interval.SetStart(v,t);break;case 1:interval.FuseAtStart(v,t);break;
  case 2:interval.CutAtStart(v,t);break;case 3:interval.SetEnd(v,t);break;
  case 4:interval.FuseAtEnd(v,t);break;default:interval.CutAtEnd(v,t);break;}
 hl_iv_print(interval);ga_flag(interval.IsProbablyEmpty());ga_end();}
// `a b`: a.Position(b) and the 13 tests of a against b.
void hl_iv_relate_case(std::istream& input){
 GaReader r{input};Intrv_Interval a=hl_iv_read(r);Intrv_Interval b=hl_iv_read(r);
 ga_int(int(a.Position(b)));
 ga_flag(a.IsBefore(b));ga_flag(a.IsAfter(b));ga_flag(a.IsInside(b));ga_flag(a.IsEnclosing(b));
 ga_flag(a.IsJustEnclosingAtStart(b));ga_flag(a.IsJustEnclosingAtEnd(b));ga_flag(a.IsJustBefore(b));
 ga_flag(a.IsJustAfter(b));ga_flag(a.IsOverlappingAtStart(b));ga_flag(a.IsOverlappingAtEnd(b));
 ga_flag(a.IsJustOverlappingAtStart(b));ga_flag(a.IsJustOverlappingAtEnd(b));ga_flag(a.IsSimilar(b));
 ga_end();}
// `op set tool`: op 0 Intersect, 1 Subtract, 2 Unite, 3 XUnite with the first interval of
// the tool, 4..7 the same with the whole tool; then the resulting set.
void hl_iv_set_case(std::istream& input){
 GaReader r{input};int op=r.integer();Intrv_Intervals set=hl_iv_read_set(r);Intrv_Intervals tool=hl_iv_read_set(r);
 GA_TRY(
  switch(op){
   case 0:set.Intersect(tool.Value(1));break;case 1:set.Subtract(tool.Value(1));break;
   case 2:set.Unite(tool.Value(1));break;case 3:set.XUnite(tool.Value(1));break;
   case 4:set.Intersect(tool);break;case 5:set.Subtract(tool);break;
   case 6:set.Unite(tool);break;default:set.XUnite(tool);break;}
  hl_iv_print_set(set))
 ga_end();}
// `set index`: Value(index), or the raised exception.
void hl_iv_value_case(std::istream& input){
 GaReader r{input};Intrv_Intervals set=hl_iv_read_set(r);int index=r.integer();
 ga_int(set.NbIntervals());GA_TRY(hl_iv_print(set.Value(index)))ga_end();}
