// The ShapeConstruct package functions; the Rust twin is
// crates/tkshhealing/examples/sh_probe/construct_package.rs.
//
// `sh_construct_pkg op ...`:
//  0 (curve record) first last tol continuity maxseg maxdeg: ConvertCurveToBSpline (3D);
//  1 (2d curve record) first last tol continuity maxseg maxdeg: ConvertCurveToBSpline (2D);
//  2 (surface) uf ul vf vl tol continuity maxseg maxdeg: ConvertSurfaceToBSpline, where a
//    surface is `0 (tp_surface record)`, `1 (curve record) direction` (an extrusion),
//    `2 (curve record) P D` (a revolution), `3 (surface) u1 u2 v1 v2` (a trimmed one) or
//    `4 (surface) u1 u2 v1 v2` (GeomConvert::SurfaceToBSplineSurface of the trimmed one);
//  3 (tp_face record) n (edge index, orientation)...: JoinPCurves of the chain on the face
//    into a new edge, then the flags, ranges and p-curves stored on it.
// Every result prints whether it is the input handle, its digest and its B-spline data.
#include <Geom_SurfaceOfLinearExtrusion.hxx>
#include <Geom_SurfaceOfRevolution.hxx>
#include <NCollection_HSequence.hxx>
#include <GeomConvert.hxx>
#include <ShapeConstruct.hxx>

void scp_bspline_surface(const occ::handle<Geom_BSplineSurface>& s){
 if(s.IsNull())return;
 ga_int(s->UDegree());ga_int(s->VDegree());ga_int(s->NbUPoles());ga_int(s->NbVPoles());
 ga_flag(s->IsUPeriodic());ga_flag(s->IsVPeriodic());ga_flag(s->IsURational());ga_flag(s->IsVRational());
 for(int i=1;i<=s->NbUKnots();++i){ga_real(s->UKnot(i));ga_int(s->UMultiplicity(i));}
 for(int i=1;i<=s->NbVKnots();++i){ga_real(s->VKnot(i));ga_int(s->VMultiplicity(i));}
 for(int i=1;i<=s->NbUPoles();++i)for(int j=1;j<=s->NbVPoles();++j){ga_pnt(s->Pole(i,j));ga_real(s->Weight(i,j));}}
occ::handle<Geom_Surface> scp_surface(GaReader& r){
 int kind=r.integer();
 if(kind==0)return tp_surface(r);
 if(kind==1){occ::handle<Geom_Curve> c=tp_geom_curve(r);return new Geom_SurfaceOfLinearExtrusion(c,r.dir());}
 if(kind==2){occ::handle<Geom_Curve> c=tp_geom_curve(r);gp_Pnt p=r.pnt();return new Geom_SurfaceOfRevolution(c,gp_Ax1(p,r.dir()));}
 occ::handle<Geom_Surface> basis=scp_surface(r);double u1=r.real(),u2=r.real(),v1=r.real();
 occ::handle<Geom_Surface> trimmed=new Geom_RectangularTrimmedSurface(basis,u1,u2,v1,r.real());
 if(kind==3)return trimmed;
 return GeomConvert::SurfaceToBSplineSurface(trimmed);}
void scp_curve(GaReader& r){
 occ::handle<Geom_Curve> c=tp_geom_curve(r);double first=r.real(),last=r.real(),tol=r.real();
 GeomAbs_Shape cont=GeomAbs_Shape(r.integer());int seg=r.integer(),deg=r.integer();
 GA_TRY(occ::handle<Geom_BSplineCurve> b=ShapeConstruct::ConvertCurveToBSpline(c,first,last,tol,cont,seg,deg);
  ga_flag(b==c);shc_curve(b);sh_construct_spline3d(b))}
void scp_curve2d(GaReader& r){
 occ::handle<Geom2d_Curve> c=tp_geom2d_curve(r);double first=r.real(),last=r.real(),tol=r.real();
 GeomAbs_Shape cont=GeomAbs_Shape(r.integer());int seg=r.integer(),deg=r.integer();
 GA_TRY(occ::handle<Geom2d_BSplineCurve> b=ShapeConstruct::ConvertCurveToBSpline(c,first,last,tol,cont,seg,deg);
  ga_flag(b==c);shc_curve2d(b);sh_construct_spline2d(b))}
void scp_surface_case(GaReader& r){
 occ::handle<Geom_Surface> s;GA_TRY(s=scp_surface(r))
 double uf=r.real(),ul=r.real(),vf=r.real(),vl=r.real(),tol=r.real();
 GeomAbs_Shape cont=GeomAbs_Shape(r.integer());int seg=r.integer(),deg=r.integer();
 if(s.IsNull())return;
 GA_TRY(occ::handle<Geom_BSplineSurface> b=ShapeConstruct::ConvertSurfaceToBSpline(s,uf,ul,vf,vl,tol,cont,seg,deg);
  shc_surface(b);scp_bspline_surface(b))}
void scp_pcurve(const TopoDS_Edge& e,const TopoDS_Face& f){
 double first=0.,last=0.;occ::handle<Geom2d_Curve> c=BRep_Tool::CurveOnSurface(e,f,first,last);
 ga_real(first);ga_real(last);shc_curve2d(c);sh_construct_spline2d(c);}
void scp_join(GaReader& r){
 TpFace t=tp_face(r);occ::handle<NCollection_HSequence<TopoDS_Shape>> chain=new NCollection_HSequence<TopoDS_Shape>();
 int n=r.integer();
 for(int k=0;k<n;++k){int i=r.integer();chain->Append(t.edges.at(i).Oriented(TopAbs_Orientation(r.integer())));}
 BRep_Builder B;TopoDS_Edge e;B.MakeEdge(e);
 ga_flag(ShapeConstruct::JoinPCurves(chain,t.face,e));
 ga_flag(BRep_Tool::SameRange(e));ga_flag(BRep_Tool::SameParameter(e));
 ga_flag(BRep_Tool::IsClosed(e,t.face));
 GA_TRY(scp_pcurve(e,t.face))
 GA_TRY(scp_pcurve(TopoDS::Edge(e.Reversed()),t.face))}
void sh_construct_pkg_case(std::istream& input){
 GaReader r{input};int op=r.integer();
 if(op==0)scp_curve(r);else if(op==1)scp_curve2d(r);else if(op==2)scp_surface_case(r);else scp_join(r);
 ga_end();}
