// HLRBRep_Surface, HLRBRep_SurfaceTool, HLRBRep_SLProps and HLRBRep_InterCSurf, the Rust
// twin of crates/tkhlr/examples/hl_probe/hlrbrep_surface.rs.
#include <HLRBRep_InterCSurf.hxx>
#include <HLRBRep_SLProps.hxx>
#include <HLRBRep_Surface.hxx>
#include <HLRBRep_SurfaceTool.hxx>
#include <HLRBRep_ThePolygonOfInterCSurf.hxx>
#include <HLRBRep_ThePolyhedronOfInterCSurf.hxx>
void hb_surface_queries(const HLRBRep_Surface& s,double tf,double tr){
 ga_int(s.GetType());GA_TRY(ga_flag(s.IsSide(tf,tr)));
 GA_TRY(gp_Pln p=s.Plane();ga_pnt(p.Location());ga_dir(p.Axis().Direction()));
 const HLRBRep_Surface* p=&s;
 GA_TRY(ga_int(HLRBRep_SurfaceTool::NbSamplesU(p));ga_int(HLRBRep_SurfaceTool::NbSamplesV(p)));
 GA_TRY(double u1=HLRBRep_SurfaceTool::FirstUParameter(p),u2=HLRBRep_SurfaceTool::LastUParameter(p);
  double v1=HLRBRep_SurfaceTool::FirstVParameter(p),v2=HLRBRep_SurfaceTool::LastVParameter(p);
  ga_int(HLRBRep_SurfaceTool::NbSamplesU(p,u1,u2));ga_int(HLRBRep_SurfaceTool::NbSamplesV(p,v1,v2));
  ga_int(HLRBRep_SurfaceTool::NbSamplesU(p,u1,(u1+u2)/3));ga_int(HLRBRep_SurfaceTool::NbSamplesV(p,v1,(v1+v2)/3)));}
void hb_slprops(const HLRBRep_Surface& s,double u,double v){
 HLRBRep_SLProps props(2,1e-9);props.SetSurface((HLRBRep_SurfacePtr)&s);
 GA_TRY(props.SetParameters(u,v);ga_pnt(props.Value());bool n=props.IsNormalDefined();ga_flag(n);
  if(n)ga_dir(props.Normal());bool c=props.IsCurvatureDefined();ga_flag(c);
  if(c){ga_real(props.MaxCurvature());ga_real(props.MinCurvature());});}
void hb_ics_print(const HLRBRep_InterCSurf& i){
 ga_flag(i.IsDone());if(!i.IsDone())return;ga_int(i.NbPoints());
 for(int k=1;k<=i.NbPoints();++k)hb_ics_point(i.Point(k));ga_int(i.NbSegments());}
// A sight line through `(x, y)` of the view against the surface: mode 0 Perform(L, S),
// mode 1 with the polygon of [pmin, pmax] and the uniform polyhedron of the surface.
void hb_ics_sight(const HLRAlgo_Projector& projector,HLRBRep_Surface& s,GaReader& r){
 double x=r.real(),y=r.real();int mode=r.integer();double pmin=r.real(),pmax=r.real();
 GA_TRY(gp_Lin l=projector.Shoot(x,y);HLRBRep_InterCSurf i;
  if(mode==0)i.Perform(l,&s);
  else{double u1=HLRBRep_SurfaceTool::FirstUParameter(&s),u2=HLRBRep_SurfaceTool::LastUParameter(&s);
   double v1=HLRBRep_SurfaceTool::FirstVParameter(&s),v2=HLRBRep_SurfaceTool::LastVParameter(&s);
   int nu=HLRBRep_SurfaceTool::NbSamplesU(&s,u1,u2),nv=HLRBRep_SurfaceTool::NbSamplesV(&s,v1,v2);
   HLRBRep_ThePolyhedronOfInterCSurf poly(&s,nu,nv,u1,v1,u2,v2);HLRBRep_ThePolygonOfInterCSurf polygon(l,pmin,pmax,3);
   ga_real(poly.DeflectionOverEstimation());i.Perform(l,polygon,&s,poly);}
  hb_ics_print(i));}
// `projector face tolf toler u1 v1 u2 v2 edge back tolc nsight (x y mode pmin pmax)*`.
void hb_surface_case(std::istream& input){
 GaReader r{input};HLRAlgo_Projector projector=hb_projector(r);
 GA_TRY(TopoDS_Face face=hb_face(r);double tf=r.real(),tr=r.real();
  double pu1=r.real(),pv1=r.real(),pu2=r.real(),pv2=r.real();TopoDS_Edge edge=hb_edge(r);
  bool back=r.flag();double tc=r.real();
  HLRBRep_Surface s;s.Projector(&projector);s.Surface(face);hb_surface_queries(s,tf,tr);
  hb_slprops(s,pu1,pv1);hb_slprops(s,pu2,pv2);
  HLRBRep_Curve c;c.Projector(&projector);c.Curve(edge);double mn[16],mx[16];HLRAlgo::InitMinMax(1e100,mn,mx);
  GA_TRY(c.Update(mn,mx);ga_flag(s.IsAbove(back,&c,tc)));
  for(int n=r.integer();n>0;--n)hb_ics_sight(projector,s,r));
 ga_end();}
