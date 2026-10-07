// The ChFi3d operations of the TKFillet probe that run without the TKBool data structure.
#include <ChFi3d_Builder_0.hxx>
#include <GC_MakeSegment2d.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Geom2dHatch_Hatcher.hxx>
#include <Geom2dHatch_Intersector.hxx>
#include <Geom2d_Line.hxx>
#include <HatchGen_Domain.hxx>
#include <gp_Ax2d.hxx>

// `fl_hatching_domains n P1..Pn P D tol keepSegments`: ChFi3d_ComputeHatchingDomains on
// the hatching line (P, D) trimmed by the closed polygon of the 2d points: the result,
// the points of the hatching (parameter, segment flags) and the domains.
void fl_hatching_domains_case(std::istream& in){
 GaReader r{in};int n=r.integer();std::vector<gp_Pnt2d> pts;
 for(int i=0;i<n;++i)pts.push_back(r.pnt2d());
 gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();double tol=r.real();bool keep=r.flag();
 Geom2dHatch_Intersector intersector(tol,tol);Geom2dHatch_Hatcher hatcher(intersector,tol,tol,false,keep);
 for(int i=0;i<n;++i)hatcher.AddElement(GC_MakeSegment2d(pts[i],pts[(i+1)%n]).Value(),TopAbs_FORWARD);
 occ::handle<Geom2d_Line> line=new Geom2d_Line(gp_Ax2d(p,d));
 int ih=hatcher.Trim(Geom2dAdaptor_Curve(line));
 GA_TRY(ga_flag(ChFi3d_ComputeHatchingDomains(hatcher,ih)));
 ga_int(hatcher.NbPoints(ih));
 for(int i=1;i<=hatcher.NbPoints(ih);++i){const HatchGen_PointOnHatching& pt=hatcher.Point(ih,i);
  ga_real(pt.Parameter());ga_flag(pt.SegmentBeginning());ga_flag(pt.SegmentEnd());}
 GA_TRY(ga_int(hatcher.NbDomains(ih));
  for(int i=1;i<=hatcher.NbDomains(ih);++i){const HatchGen_Domain& dom=hatcher.Domain(ih,i);
   ga_flag(dom.HasFirstPoint());if(dom.HasFirstPoint())ga_real(dom.FirstPoint().Parameter());
   ga_flag(dom.HasSecondPoint());if(dom.HasSecondPoint())ga_real(dom.SecondPoint().Parameter());});
 ga_end();}
