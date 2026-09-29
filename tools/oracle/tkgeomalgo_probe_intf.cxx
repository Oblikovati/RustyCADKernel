// Intf probe operations over the package functions and the result records (section
// points, section lines, tangent zones, Intf_Interference); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/intf.rs.
#include <Intf.hxx>
#include <Intf_Interference.hxx>
#include <Intf_SectionLine.hxx>
#include <Intf_SectionPoint.hxx>
#include <Intf_TangentZone.hxx>
// `p1 p2 p3 point`: the plane equation of the triangle, then whether it contains the point.
void ga_intf_triangle_case(std::istream& input){
 GaReader r{input};gp_Pnt p1=r.pnt(),p2=r.pnt(),p3=r.pnt(),point=r.pnt();
 gp_XYZ normal(-1.,-2.,-3.);double distance=-4.;
 Intf::PlaneEquation(p1,p2,p3,normal,distance);
 ga_xyz(normal);ga_real(distance);ga_flag(Intf::Contain(p1,p2,p3,point));ga_end();
}
// A section point: `0 x y z dimO o1 o2 parO dimT t1 t2 parT incid` or
// `1 x y dimO o parO dimT t parT incid` (the 2d constructor).
Intf_SectionPoint ga_spoint(GaReader& r){
 if(r.integer()==0){gp_Pnt p=r.pnt();Intf_PIType d1=Intf_PIType(r.integer());int a1=r.integer(),a2=r.integer();double p1=r.real();
  Intf_PIType d2=Intf_PIType(r.integer());int a3=r.integer(),a4=r.integer();double p2=r.real();
  return Intf_SectionPoint(p,d1,a1,a2,p1,d2,a3,a4,p2,r.real());}
 gp_Pnt2d p=r.pnt2d();Intf_PIType d1=Intf_PIType(r.integer());int a1=r.integer();double p1=r.real();
 Intf_PIType d2=Intf_PIType(r.integer());int a2=r.integer();double p2=r.real();
 return Intf_SectionPoint(p,d1,a1,p1,d2,a2,p2,r.real());
}
std::vector<Intf_SectionPoint> ga_spoints(GaReader& r){
 int n=r.integer();std::vector<Intf_SectionPoint> points;for(int i=0;i<n;++i)points.push_back(ga_spoint(r));return points;}
// Every accessor of a section point.
void ga_spoint_print(const Intf_SectionPoint& p){
 ga_pnt(p.Pnt());Intf_PIType d;int a1,a2;double par;
 p.InfoFirst(d,a1,a2,par);ga_int(d);ga_int(a1);ga_int(a2);ga_real(par);
 p.InfoFirst(d,a1,par);ga_int(d);ga_int(a1);ga_real(par);
 p.InfoSecond(d,a1,a2,par);ga_int(d);ga_int(a1);ga_int(a2);ga_real(par);
 p.InfoSecond(d,a1,par);ga_int(d);ga_int(a1);ga_real(par);
 ga_real(p.ParamOnFirst());ga_real(p.ParamOnSecond());ga_int(p.TypeOnFirst());ga_int(p.TypeOnSecond());ga_real(p.Incidence());
}
// `a b`: both points, the comparisons both ways, then both after a.Merge(b).
void ga_intf_spoint_case(std::istream& input){
 GaReader r{input};Intf_SectionPoint a=ga_spoint(r),b=ga_spoint(r);
 ga_spoint_print(Intf_SectionPoint());ga_spoint_print(a);ga_spoint_print(b);
 ga_flag(a.IsEqual(b));ga_flag(a==b);ga_flag(a.IsOnSameEdge(b));ga_flag(b.IsOnSameEdge(a));
 a.Merge(b);ga_spoint_print(a);ga_spoint_print(b);ga_end();
}
void ga_sline_print(const Intf_SectionLine& line){
 ga_int(line.NumberOfPoints());
 for(int i=1;i<=line.NumberOfPoints();++i)ga_spoint_print(line.GetPoint(i));
}
Intf_SectionLine ga_sline_of(GaReader& r,const std::vector<Intf_SectionPoint>& points){
 Intf_SectionLine line;int n=r.integer();for(int i=0;i<n;++i)line.Append(points[r.integer()]);return line;}
// `points commands`: Append/Prepend a point (0/1 k), Reverse (2), Close (3), Append/Prepend
// a line of point indices (4/5 m k...), IsEnd (6 k), Contains (7 k), IsClosed (8),
// GetPoint (9 i), Dump (10 indent), copy and IsEqual (11), assignment (12 m k...).
void ga_sline_command(GaReader& r,Intf_SectionLine& line,const std::vector<Intf_SectionPoint>& p,int command){
 switch(command){
  case 0:line.Append(p[r.integer()]);break;
  case 1:line.Prepend(p[r.integer()]);break;
  case 2:line.Reverse();break;
  case 3:line.Close();break;
  case 4:{Intf_SectionLine other=ga_sline_of(r,p);line.Append(other);ga_int(other.NumberOfPoints());break;}
  case 5:{Intf_SectionLine other=ga_sline_of(r,p);line.Prepend(other);ga_int(other.NumberOfPoints());break;}
  case 6:{const Intf_SectionPoint& k=p[r.integer()];GA_TRY(ga_int(line.IsEnd(k)));break;}
  case 7:ga_flag(line.Contains(p[r.integer()]));break;
  case 8:GA_TRY(ga_flag(line.IsClosed()));break;
  case 9:{int i=r.integer();GA_TRY(ga_spoint_print(line.GetPoint(i)));break;}
  case 10:{int indent=r.integer();ga_captured([&]{line.Dump(indent);});break;}
  case 11:{Intf_SectionLine copy(line);ga_flag(copy.IsEqual(line));ga_flag(copy==line);ga_sline_print(copy);break;}
  default:{Intf_SectionLine other=ga_sline_of(r,p);ga_flag(other==line);other=line;ga_flag(other.IsEqual(line));break;}
 }
}
void ga_intf_sline_case(std::istream& input){
 GaReader r{input};std::vector<Intf_SectionPoint> points=ga_spoints(r);Intf_SectionLine line;int count=r.integer();
 for(int k=0;k<count;++k){ga_sline_command(r,line,points,r.integer());ga_int(line.NumberOfPoints());}
 ga_sline_print(line);ga_end();
}
void ga_tzone_print(const Intf_TangentZone& zone){
 ga_int(zone.NumberOfPoints());double a,b;zone.ParamOnFirst(a,b);ga_real(a);ga_real(b);zone.ParamOnSecond(a,b);ga_real(a);ga_real(b);
 int s0,s1;zone.InfoFirst(s0,a,s1,b);ga_int(s0);ga_real(a);ga_int(s1);ga_real(b);
 zone.InfoSecond(s0,a,s1,b);ga_int(s0);ga_real(a);ga_int(s1);ga_real(b);
 for(int i=1;i<=zone.NumberOfPoints();++i)ga_spoint_print(zone.GetPoint(i));
}
Intf_TangentZone ga_tzone_of(GaReader& r,const std::vector<Intf_SectionPoint>& points){
 Intf_TangentZone zone;int n=r.integer();for(int i=0;i<n;++i)zone.Append(points[r.integer()]);return zone;}
// `points commands`: Append (0 k), PolygonInsert (1 k), InsertBefore/After (2/3 i k),
// Insert (4 k), Append of a zone (5 m k...), Contains (6 k), RangeContains (7 k),
// GetPoint (8 i), HasCommonRange both ways (9 m k...), IsEqual with a copy (10).
void ga_tzone_command(GaReader& r,Intf_TangentZone& zone,const std::vector<Intf_SectionPoint>& p,int command){
 switch(command){
  case 0:zone.Append(p[r.integer()]);break;
  case 1:zone.PolygonInsert(p[r.integer()]);break;
  case 2:{int i=r.integer();const Intf_SectionPoint& k=p[r.integer()];GA_TRY(zone.InsertBefore(i,k));break;}
  case 3:{int i=r.integer();const Intf_SectionPoint& k=p[r.integer()];GA_TRY(zone.InsertAfter(i,k));break;}
  case 4:ga_flag(zone.Insert(p[r.integer()]));break;
  case 5:{Intf_TangentZone other=ga_tzone_of(r,p);zone.Append(other);break;}
  case 6:ga_flag(zone.Contains(p[r.integer()]));break;
  case 7:ga_flag(zone.RangeContains(p[r.integer()]));break;
  case 8:{int i=r.integer();GA_TRY(ga_spoint_print(zone.GetPoint(i)));break;}
  case 9:{Intf_TangentZone other=ga_tzone_of(r,p);ga_flag(zone.HasCommonRange(other));ga_flag(other.HasCommonRange(zone));break;}
  default:{Intf_TangentZone copy(zone);ga_flag(copy.IsEqual(zone));ga_flag(copy==zone);break;}
 }
}
void ga_intf_tzone_case(std::istream& input){
 GaReader r{input};std::vector<Intf_SectionPoint> points=ga_spoints(r);Intf_TangentZone zone;int count=r.integer();
 for(int k=0;k<count;++k){ga_tzone_command(r,zone,points,r.integer());ga_int(zone.NumberOfPoints());}
 ga_tzone_print(zone);ga_end();
}
// Intf_Interference has a protected constructor and members: the probe derives from it.
struct GaInterference : public Intf_Interference{
 GaInterference():Intf_Interference(false){}
 void AddZone(const Intf_TangentZone& zone){myTZones.Append(zone);}
 void AddPoint(const Intf_SectionPoint& point){mySPoins.Append(point);}
 void SetTolerance(double tolerance){Tolerance=tolerance;}
 void Reset(bool self){SelfInterference(self);}
};
void ga_interference_print(const Intf_Interference& interference){
 ga_real(interference.GetTolerance());ga_int(interference.NbSectionPoints());
 for(int i=1;i<=interference.NbSectionPoints();++i)ga_spoint_print(interference.PntValue(i));
 ga_int(interference.NbSectionLines());
 for(int i=1;i<=interference.NbSectionLines();++i)ga_sline_print(interference.LineValue(i));
 ga_int(interference.NbTangentZones());
 for(int i=1;i<=interference.NbTangentZones();++i)ga_tzone_print(interference.ZoneValue(i));
 GA_TRY(interference.PntValue(interference.NbSectionPoints()+1));
 GA_TRY(interference.LineValue(0));GA_TRY(interference.ZoneValue(interference.NbTangentZones()+1));
}
// `points commands`: add a zone (0 m k...), Insert a zone (1 m k...), Insert a segment
// (2 a b), Contains (3 k), Dump (4), add a section point (5 k), SelfInterference (6 flag),
// set the tolerance (7 value).
void ga_interference_command(GaReader& r,GaInterference& interference,const std::vector<Intf_SectionPoint>& p,int command){
 switch(command){
  case 0:interference.AddZone(ga_tzone_of(r,p));break;
  case 1:{Intf_TangentZone zone=ga_tzone_of(r,p);GA_TRY(ga_flag(interference.Insert(zone)));break;}
  case 2:{const Intf_SectionPoint& a=p[r.integer()];const Intf_SectionPoint& b=p[r.integer()];GA_TRY(interference.Insert(a,b));break;}
  case 3:ga_flag(interference.Contains(p[r.integer()]));break;
  case 4:ga_captured([&]{interference.Dump();});break;
  case 5:interference.AddPoint(p[r.integer()]);break;
  case 6:interference.Reset(r.flag());break;
  default:interference.SetTolerance(r.real());break;
 }
}
void ga_intf_interference_case(std::istream& input){
 GaReader r{input};std::vector<Intf_SectionPoint> points=ga_spoints(r);GaInterference interference;int count=r.integer();
 for(int k=0;k<count;++k){ga_interference_command(r,interference,points,r.integer());ga_int(interference.NbTangentZones());}
 ga_interference_print(interference);ga_end();
}
