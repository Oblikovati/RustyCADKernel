// HatchGen probe operations (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/hatch_gen.rs. IntRes2d points are read by
// ga_read_ipoint (tkgeomalgo_probe_intres2d.cxx); the dumps are captured text. The point
// printers serve the Geom2dHatch probe too.
#include <HatchGen_Domain.hxx>
#include <HatchGen_PointOnElement.hxx>
#include <HatchGen_PointOnHatching.hxx>
void ga_hgen_base(const HatchGen_IntersectionPoint& p){
 ga_int(p.Index());ga_real(p.Parameter());ga_int(p.Position());ga_int(p.StateBefore());ga_int(p.StateAfter());
 ga_flag(p.SegmentBeginning());ga_flag(p.SegmentEnd());
}
void ga_hgen_pe(const HatchGen_PointOnElement& p){ga_hgen_base(p);ga_int(p.IntersectionType());}
// A point on hatching and every point on element it holds.
void ga_hgen_ph(const HatchGen_PointOnHatching& p){
 ga_hgen_base(p);ga_int(p.NbPoints());for(int i=1;i<=p.NbPoints();i++)ga_hgen_pe(p.Point(i));
}
// `field value`: one setter of a point (0 none, 1 index, 2 parameter, 3 position, 4 state
// before, 5 state after, 6 segment beginning, 7 segment end).
void ga_hgen_set(HatchGen_IntersectionPoint& p,int field,double v){
 if(field==1)p.SetIndex(int(v));else if(field==2)p.SetParameter(v);else if(field==3)p.SetPosition(TopAbs_Orientation(int(v)));
 else if(field==4)p.SetStateBefore(TopAbs_State(int(v)));else if(field==5)p.SetStateAfter(TopAbs_State(int(v)));
 else if(field==6)p.SetSegmentBeginning(v!=0.);else if(field==7)p.SetSegmentEnd(v!=0.);
}
void ga_hgen_dump(const HatchGen_IntersectionPoint& p,int index){ga_captured([&]{p.Dump(index);});}
// `point index confusion field value type`: the point on element of an IntRes2d point, its
// copy after one setter (and a type), both comparisons both ways, the dumps; then the
// default point.
void ga_hgen_element_case(std::istream& input){
 GaReader r{input};IntRes2d_IntersectionPoint ip=ga_read_ipoint(r);int index=r.integer();double conf=r.real();
 int field=r.integer();double v=r.real();int type=r.integer();
 GA_TRY(HatchGen_PointOnElement p(ip);ga_hgen_pe(p);p.SetIndex(index);HatchGen_PointOnElement q(p);ga_hgen_set(q,field,v);
  if(type>=0)q.SetIntersectionType(HatchGen_IntersectionType(type));ga_hgen_pe(q);
  ga_flag(p.IsIdentical(q,conf));ga_flag(p.IsDifferent(q,conf));ga_flag(q.IsIdentical(p,conf));ga_flag(q.IsDifferent(p,conf));
  ga_hgen_dump(p,index);ga_hgen_dump(q,0));
 HatchGen_PointOnElement d;ga_hgen_pe(d);ga_hgen_dump(d,-1);ga_end();
}
// `point n (point index field value)*n confusion other rem`: a point on hatching with the
// points on element added under `confusion`, its accessors from 0 to NbPoints + 1, the
// order tests against the point on hatching of `other`, the dump, RemPoint(rem) and
// ClrPoints.
void ga_hgen_hatching_case(std::istream& input){
 GaReader r{input};IntRes2d_IntersectionPoint ip=ga_read_ipoint(r);int n=r.integer();
 std::vector<HatchGen_PointOnElement> elements;std::string error;
 for(int k=0;k<n;k++){IntRes2d_IntersectionPoint e=ga_read_ipoint(r);int index=r.integer(),field=r.integer();double v=r.real();
  try{HatchGen_PointOnElement pe(e);pe.SetIndex(index);ga_hgen_set(pe,field,v);elements.push_back(pe);}
  catch(const Standard_Failure& f){std::cout<<f.ExceptionType()<<' ';}}
 double conf=r.real();IntRes2d_IntersectionPoint other=ga_read_ipoint(r);int rem=r.integer();
 HatchGen_PointOnHatching p(ip);for(const HatchGen_PointOnElement& pe:elements)p.AddPoint(pe,conf);
 ga_hgen_ph(p);for(int i=0;i<=p.NbPoints()+1;i++)GA_TRY(ga_hgen_pe(p.Point(i)));
 HatchGen_PointOnHatching o(other);ga_flag(p.IsLower(o,conf));ga_flag(p.IsEqual(o,conf));ga_flag(p.IsGreater(o,conf));
 ga_flag(o.IsLower(p,conf));ga_flag(o.IsEqual(p,conf));ga_flag(o.IsGreater(p,conf));ga_hgen_dump(p,n);
 GA_TRY(p.RemPoint(rem));ga_hgen_ph(p);p.ClrPoints();ga_hgen_ph(p);ga_hgen_dump(p,0);
 HatchGen_PointOnHatching d;ga_hgen_ph(d);ga_hgen_dump(d,0);ga_end();
}
void ga_hgen_domain_out(const HatchGen_Domain& d){
 ga_flag(d.HasFirstPoint());GA_TRY(ga_hgen_ph(d.FirstPoint()));ga_flag(d.HasSecondPoint());GA_TRY(ga_hgen_ph(d.SecondPoint()));
}
// `point1 point2 n op*n index`: the four constructors, then a domain changed by each op (0
// SetPoints(P1, P2), 1 SetPoints(), 2 SetFirstPoint(P1), 3 SetFirstPoint(), 4
// SetSecondPoint(P2), 5 SetSecondPoint()) and its dump with `index`.
void ga_hgen_domain_case(std::istream& input){
 GaReader r{input};HatchGen_PointOnHatching p1(ga_read_ipoint(r)),p2(ga_read_ipoint(r));p1.SetIndex(1);p2.SetIndex(2);
 HatchGen_PointOnElement pe;pe.SetIndex(7);p2.AddPoint(pe,0.);
 HatchGen_Domain d;ga_hgen_domain_out(d);ga_hgen_domain_out(HatchGen_Domain(p1,p2));
 ga_hgen_domain_out(HatchGen_Domain(p1,true));ga_hgen_domain_out(HatchGen_Domain(p2,false));
 int n=r.integer();
 for(int k=0;k<n;k++){int op=r.integer();
  if(op==0)d.SetPoints(p1,p2);else if(op==1)d.SetPoints();else if(op==2)d.SetFirstPoint(p1);
  else if(op==3)d.SetFirstPoint();else if(op==4)d.SetSecondPoint(p2);else d.SetSecondPoint();
  ga_hgen_domain_out(d);}
 int index=r.integer();ga_captured([&]{d.Dump(index);});ga_end();
}
