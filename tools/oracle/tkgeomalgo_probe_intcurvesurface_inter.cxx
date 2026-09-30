// IntCurveSurface probe operations over the interference, the CS function and exact
// intersector, the quadric intersector, IntCurveSurface_HInter (its protected members
// through a probe subclass) and the IntCurveSurface_InterUtils functions
// (tkgeomalgo_probe.cxx); the Rust twin is
// crates/tkgeomalgo/examples/ga_probe/int_curve_surface_inter.rs.
#include <Adaptor3d_HSurfaceTool.hxx>
#include <IntCurveSurface_HInter.hxx>
#include <IntCurveSurface_TheCSFunctionOfHInter.hxx>
#include <IntCurveSurface_TheExactHInter.hxx>
#include <IntCurveSurface_TheInterferenceOfHInter.hxx>
#include <IntCurveSurface_TheQuadCurvExactHInter.hxx>
#include <IntCurveSurface_TheQuadCurvFuncOfTheQuadCurvExactHInter.hxx>
#include <IntCurveSurface_InterUtils.pxx>
#include <IntAna_IntConicQuad.hxx>
#include <IntAna_Quadric.hxx>
#include <ProjLib_Plane.hxx>
Intf_SectionPoint ga_spoint(GaReader& r);
void ga_interference_print(const Intf_Interference& interference);
NCollection_Array1<gp_Lin> ga_lines(GaReader& r);
IntSurf_Quadric ga_intsurf_quadric(GaReader& r);
// `curve polygon surface polyhedron count (kind operand)...`: the interference members:
// 0..2 Perform with the polygon, a line (`p d`) or lines, 3..5 the same with the grid of the
// polyhedron, 6 and 7 Interference without and with the grid, 8..13 the constructors.
void ga_ics_interference_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Curve> c=ga_ics_curve(r);IntCurveSurface_ThePolygonOfHInter pg=ga_ics_polygon(r,c);
 occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);IntCurveSurface_ThePolyhedronOfHInter ph=ga_ics_polyhedron(r,s);
 IntCurveSurface_TheInterferenceOfHInter x;int count=r.integer();using X=IntCurveSurface_TheInterferenceOfHInter;
 for(int k=0;k<count;++k){int kind=r.integer();int base=kind>=8?kind-8:kind;bool make=kind>=8;
  Bnd_BoundSortBox grid;grid.Initialize(ph.Bounding(),ph.ComponentsBounding());
  if(kind==6)GA_TRY(x.Interference(pg,ph))else if(kind==7)GA_TRY(x.Interference(pg,ph,grid))
  else if(base==0)GA_TRY(if(make)x=X(pg,ph);else x.Perform(pg,ph))
  else if(base==3)GA_TRY(if(make)x=X(pg,ph,grid);else x.Perform(pg,ph,grid))
  else if(base==1||base==4){gp_Pnt p=r.pnt();gp_Lin l(p,r.dir());
   if(base==1)GA_TRY(if(make)x=X(l,ph);else x.Perform(l,ph))else GA_TRY(if(make)x=X(l,ph,grid);else x.Perform(l,ph,grid))}
  else{NCollection_Array1<gp_Lin> ls=ga_lines(r);
   if(base==2)GA_TRY(if(make)x=X(ls,ph);else x.Perform(ls,ph))else GA_TRY(if(make)x=X(ls,ph,grid);else x.Perform(ls,ph,grid))}
  ga_interference_print(x);}
 ga_end();
}
void ga_ics_exact_print(IntCurveSurface_TheExactHInter& e){
 ga_flag(e.IsDone());GA_TRY(ga_flag(e.IsEmpty()));GA_TRY(ga_pnt(e.Point()));GA_TRY(double a,b;e.ParameterOnSurface(a,b);ga_real(a);ga_real(b));
 GA_TRY(ga_real(e.ParameterOnCurve()));ga_real(e.Function().Root());}
// `surface curve u v w tol margin bounds(6)`: the CS function at (u, v, w), the exact
// intersector from the point, and Perform of a bare intersector within the bounds.
void ga_ics_exact_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);occ::handle<Adaptor3d_Curve> c=ga_ics_curve(r);
 double u=r.real(),v=r.real(),w=r.real(),tol=r.real(),margin=r.real();double b[6];for(double& x:b)x=r.real();
 GA_TRY(IntCurveSurface_TheCSFunctionOfHInter f(s,c);ga_int(f.NbVariables());ga_int(f.NbEquations());math_Vector x(1,3);x(1)=u;x(2)=v;x(3)=w;
  math_Vector fv(1,3,0.0);math_Matrix d(1,3,1,3,0.0);GA_TRY(ga_flag(f.Value(x,fv));ga_ii_vector(fv);ga_pnt(f.Point());ga_real(f.Root()));
  GA_TRY(ga_flag(f.Derivatives(x,d));ga_ii_matrix(d));x(1)+=0.01;GA_TRY(ga_flag(f.Values(x,fv,d));ga_ii_vector(fv);ga_ii_matrix(d);ga_real(f.Root()));
  ga_real(f.AuxillarSurface()->FirstUParameter());ga_real(f.AuxillarCurve()->LastParameter());
  GA_TRY(IntCurveSurface_TheExactHInter e(u,v,w,f,tol,margin);ga_ics_exact_print(e));
  GA_TRY(IntCurveSurface_TheExactHInter e(f,tol);ga_flag(e.IsDone());GA_TRY(ga_flag(e.IsEmpty()));math_FunctionSetRoot rs(e.Function());
   e.Perform(u,v,w,rs,b[0],b[1],b[2],b[3],b[4],b[5]);ga_ics_exact_print(e)));
 ga_end();
}
// `surface curve quadric n params...`: IntCurveSurface_TheQuadCurvExactHInter, then the
// distance function on the quadric at each parameter.
void ga_ics_quadcurv_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);occ::handle<Adaptor3d_Curve> c=ga_ics_curve(r);
 IntSurf_Quadric q=ga_intsurf_quadric(r);NCollection_Array1<double> pars=ga_ics_params(r);
 GA_TRY(IntCurveSurface_TheQuadCurvExactHInter e(s,c);ga_flag(e.IsDone());int n=e.NbRoots(),m=e.NbIntervals();ga_int(n);ga_int(m);
  for(int i=0;i<=std::max(n,0)+1;++i)GA_TRY(ga_real(e.Root(i)));
  for(int i=0;i<=std::max(m,0)+1;++i)GA_TRY(double a,b;e.Intervals(i,a,b);ga_real(a);ga_real(b)));
 IntCurveSurface_TheQuadCurvFuncOfTheQuadCurvExactHInter f(q,c);
 for(double t:pars){GA_TRY(double y=-7;ga_flag(f.Value(t,y));ga_real(y));GA_TRY(double y=-7;ga_flag(f.Derivative(t,y));ga_real(y));
  GA_TRY(double y=-7,z=-8;ga_flag(f.Values(t,y,z));ga_real(y);ga_real(z));}
 ga_end();
}
// IntCurveSurface_HInter with its protected members public, for the probe.
class GaIcsHInter : public IntCurveSurface_HInter{
public:
 using IntCurveSurface_HInter::Perform;using IntCurveSurface_HInter::InternalPerformCurveQuadric;
 using IntCurveSurface_HInter::InternalPerform;using IntCurveSurface_HInter::PerformConicSurf;
 using IntCurveSurface_HInter::AppendIntAna;using IntCurveSurface_HInter::AppendPoint;using IntCurveSurface_HInter::AppendSegment;
 void SetDone(){done=true;}
};
// `curve surface mode ...`: HInter::Perform (0), with a polygon (1), with a polygon and a
// polyhedron (2), with them and a grid (3), with a polyhedron (4).
void ga_ics_hinter_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Curve> c=ga_ics_curve(r);occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);
 GaIcsHInter x;int mode=r.integer();
 GA_TRY(if(mode==0)x.Perform(c,s);
  else if(mode==4){IntCurveSurface_ThePolyhedronOfHInter ph=ga_ics_polyhedron(r,s);x.Perform(c,s,ph);}
  else{IntCurveSurface_ThePolygonOfHInter pg=ga_ics_polygon(r,c);if(mode==1)x.Perform(c,pg,s);
   else{IntCurveSurface_ThePolyhedronOfHInter ph=ga_ics_polyhedron(r,s);if(mode==2)x.Perform(c,pg,s,ph);
    else{Bnd_BoundSortBox grid;grid.Initialize(ph.Bounding(),ph.ComponentsBounding());x.Perform(c,pg,s,ph,grid);}}});
 ga_ics_intersection_print(x);ga_end();
}
// A conic `kind p d a b`: 0 line, 1 circle, 2 ellipse, 3 parabola, 4 hyperbola.
void ga_ics_conic_surf(GaReader& r,GaIcsHInter& x,const occ::handle<Adaptor3d_Curve>& c,const occ::handle<Adaptor3d_Surface>& s){
 int k=r.integer();gp_Pnt p=r.pnt();gp_Dir d=r.dir();double a=r.real(),b=r.real(),u1=r.real(),v1=r.real(),u2=r.real(),v2=r.real();gp_Ax2 ax(p,d);
 if(k==0)x.PerformConicSurf(gp_Lin(p,d),c,s,u1,v1,u2,v2);else if(k==1)x.PerformConicSurf(gp_Circ(ax,a),c,s,u1,v1,u2,v2);
 else if(k==2)x.PerformConicSurf(gp_Elips(ax,a,b),c,s,u1,v1,u2,v2);else if(k==3)x.PerformConicSurf(gp_Parab(ax,a),c,s,u1,v1,u2,v2);
 else x.PerformConicSurf(gp_Hypr(ax,a,b),c,s,u1,v1,u2,v2);
}
// An analytic intersection `line(p d) kind ...`: 0 a plane `p d tolang`, 1 a sphere `p r`.
IntAna_IntConicQuad ga_ics_int_ana(GaReader& r){
 gp_Pnt p=r.pnt();gp_Lin l(p,r.dir());int k=r.integer();gp_Pnt q=r.pnt();
 if(k==0){gp_Pln pl(q,r.dir());return IntAna_IntConicQuad(l,pl,r.real());}
 return IntAna_IntConicQuad(l,IntAna_Quadric(gp_Sphere(gp_Ax3(q,gp_Dir(0,0,1)),r.real())));
}
// `curve surface op ...` on a done HInter: 0 Perform(u1 v1 u2 v2), 1 InternalPerformCurveQuadric,
// 2 InternalPerform(polygon polyhedron u1 v1 u2 v2), 3 the same with a grid, 4 InternalPerform
// (polygon u1 v1 u2 v2), 5 PerformConicSurf(conic u1 v1 u2 v2), 6 AppendIntAna, 7 AppendPoint
// (w u v), 8 AppendSegment (u0 u1).
void ga_ics_hinter_protected_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Curve> c=ga_ics_curve(r);occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);
 GaIcsHInter x;x.SetDone();int op=r.integer();
 GA_TRY(if(op==0){double u1=r.real(),v1=r.real(),u2=r.real();x.Perform(c,s,u1,v1,u2,r.real());}
  else if(op==1)x.InternalPerformCurveQuadric(c,s);
  else if(op==2||op==3||op==4){IntCurveSurface_ThePolygonOfHInter pg=ga_ics_polygon(r,c);
   if(op==4){double u1=r.real(),v1=r.real(),u2=r.real();x.InternalPerform(c,pg,s,u1,v1,u2,r.real());}
   else{IntCurveSurface_ThePolyhedronOfHInter ph=ga_ics_polyhedron(r,s);double u1=r.real(),v1=r.real(),u2=r.real(),v2=r.real();
    if(op==2)x.InternalPerform(c,pg,s,ph,u1,v1,u2,v2);
    else{Bnd_BoundSortBox grid;grid.Initialize(ph.Bounding(),ph.ComponentsBounding());x.InternalPerform(c,pg,s,ph,u1,v1,u2,v2,grid);}}}
  else if(op==5)ga_ics_conic_surf(r,x,c,s);
  else if(op==6)x.AppendIntAna(c,s,ga_ics_int_ana(r));
  else if(op==7){double w=r.real(),u=r.real();x.AppendPoint(c,w,s,u,r.real());}
  else{double a=r.real();x.AppendSegment(c,a,r.real(),s);});
 ga_ics_intersection_print(x);ga_end();
}
// `surface line op flags(4) bounds(u1 u2 v1 v2) nbsu offsurf`: EstLimForInfExtr (0),
// EstLimForInfRevl (1), EstLimForInfOffs (2), EstLimForInfSurf (3); prints NoIntersection
// and the new bounds.
void ga_ics_estlim_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);gp_Pnt p=r.pnt();gp_Lin l(p,r.dir());int op=r.integer();
 bool f[4];for(bool& x:f)x=r.flag();double b[4];for(double& x:b)x=r.real();int nbsu=r.integer();bool off=r.flag();
 namespace IU=IntCurveSurface_InterUtils;using S=occ::handle<Adaptor3d_Surface>;bool no=false;
 GA_TRY(if(op==0)IU::EstLimForInfExtr<S,Adaptor3d_HSurfaceTool>(l,s,off,nbsu,f[0],f[1],f[2],f[3],b[0],b[1],b[2],b[3],no);
  else if(op==1)IU::EstLimForInfRevl<S,Adaptor3d_HSurfaceTool>(l,s,f[0],f[1],f[2],f[3],b[0],b[1],b[2],b[3],no);
  else if(op==2)IU::EstLimForInfOffs<S,Adaptor3d_HSurfaceTool>(l,s,nbsu,f[0],f[1],f[2],f[3],b[0],b[1],b[2],b[3],no);
  else IU::EstLimForInfSurf(b[0],b[1],b[2],b[3]));
 ga_flag(no);for(double x:b)ga_real(x);ga_end();
}
// `line(p d) plane(p d) basis-plane(p d) conic(kind p d a b) vmin vmax`: ProjectIntersectAndEstLim
// on the projection of a line (0), parabola (3) or hyperbola (4) basis curve.
void ga_ics_project_case(std::istream& input){
 GaReader r{input};gp_Pnt p=r.pnt();gp_Lin l(p,r.dir());gp_Pnt q=r.pnt();gp_Pln pl(q,r.dir());gp_Pnt q2=r.pnt();gp_Pln bpl(q2,r.dir());
 int k=r.integer();gp_Pnt o=r.pnt();gp_Dir d=r.dir();double a=r.real(),b=r.real();double vmin=r.real(),vmax=r.real();bool no=false;
 GA_TRY(ProjLib_Plane proj(bpl);gp_Ax2 ax(o,d);if(k==0)proj.Project(gp_Lin(o,d));else if(k==3)proj.Project(gp_Parab(ax,a));
  else if(k==4)proj.Project(gp_Hypr(ax,a,b));else proj.Project(gp_Circ(ax,a));
  IntCurveSurface_InterUtils::ProjectIntersectAndEstLim(l,pl,proj,vmin,vmax,no));
 ga_flag(no);ga_real(vmin);ga_real(vmax);ga_end();
}
void ga_ics_points_print(const NCollection_DynamicArray<IntCurveSurface_IntersectionPoint>& pts){
 ga_int(int(pts.Length()));for(const IntCurveSurface_IntersectionPoint& p:pts)ga_ics_ipoint(p);}
void ga_ics_start_print(const IntCurveSurface_InterUtils::SortedStartPoints& s){
 ga_int(s.Size());for(int i=0;i<s.Size();++i){ga_real(s.TabU(i));ga_real(s.TabV(i));ga_real(s.TabW(i));}}
// The line-surface analytic intersection of the curve's line with a plane, cylinder,
// cone or sphere surface (1e-12 against a plane).
IntAna_IntConicQuad ga_ics_line_quadric(const occ::handle<Adaptor3d_Curve>& c,const occ::handle<Adaptor3d_Surface>& s){
 gp_Lin l=c->Line();switch(s->GetType()){
  case GeomAbs_Plane:return IntAna_IntConicQuad(l,s->Plane(),1e-12);
  case GeomAbs_Cylinder:return IntAna_IntConicQuad(l,IntAna_Quadric(s->Cylinder()));
  case GeomAbs_Cone:return IntAna_IntConicQuad(l,IntAna_Quadric(s->Cone()));
  default:return IntAna_IntConicQuad(l,IntAna_Quadric(s->Sphere()));}
}
// `u0 u1 v0 v1 x(2) y(2) z(2)`: DoSurface (gap, box, six samples), then DoNewBounds.
void ga_ics_sampling(GaReader& r,const occ::handle<Adaptor3d_Surface>& s){
 namespace IU=IntCurveSurface_InterUtils;using S=occ::handle<Adaptor3d_Surface>;using ST=Adaptor3d_HSurfaceTool;
 double u0=r.real(),u1=r.real(),v0=r.real(),v1=r.real();NCollection_Array1<double> x(1,2),y(1,2),z(1,2),b(1,4);
 for(NCollection_Array1<double>* a:{&x,&y,&z}){(*a)(1)=r.real();(*a)(2)=r.real();}
 NCollection_Array2<gp_Pnt> pts(1,50,1,50);Bnd_Box box;double gap=-1;IU::DoSurface<S,ST>(s,u0,u1,v0,v1,pts,box,gap);
 ga_real(gap);ga_box3d_print(box);for(auto [i,j]:{std::pair{1,1},{1,50},{25,25},{13,37},{50,1},{50,50}})ga_pnt(pts(i,j));
 IU::DoNewBounds<S,ST>(s,u0,u1,v0,v1,pts,x,y,z,b);for(double v:b)ga_real(v);
}
// `polygon polyhedron u0 u1 v0 v1`: the start points of the interference (collected, then
// sorted) and the points ProcessSortedPoints keeps.
void ga_ics_refine(GaReader& r,const occ::handle<Adaptor3d_Curve>& c,const occ::handle<Adaptor3d_Surface>& s){
 namespace IU=IntCurveSurface_InterUtils;using C=occ::handle<Adaptor3d_Curve>;using S=occ::handle<Adaptor3d_Surface>;
 IntCurveSurface_ThePolygonOfHInter pg=ga_ics_polygon(r,c);IntCurveSurface_ThePolyhedronOfHInter ph=ga_ics_polyhedron(r,s);
 double u0=r.real(),u1=r.real(),v0=r.real(),v1=r.real();IntCurveSurface_TheInterferenceOfHInter itf(pg,ph);IU::SortedStartPoints sp;
 IU::CollectInterferencePoints<IntCurveSurface_TheInterferenceOfHInter,IntCurveSurface_ThePolyhedronOfHInter,IntCurveSurface_ThePolygonOfHInter>(itf,ph,pg,sp);
 ga_ics_start_print(sp);IU::SortStartPoints(sp);ga_ics_start_print(sp);
 IntCurveSurface_TheCSFunctionOfHInter f(s,c);IntCurveSurface_TheExactHInter e(f,1e-8);math_FunctionSetRoot rs(e.Function());
 NCollection_DynamicArray<IntCurveSurface_IntersectionPoint> pts;
 IU::ProcessSortedPoints<IntCurveSurface_TheExactHInter,C,IntCurveSurface_TheHCurveTool,S,Adaptor3d_HSurfaceTool>(
  e,rs,sp,u0,u1,v0,v1,pg.InfParameter(),pg.SupParameter(),c,s,pts);
 ga_ics_points_print(pts);
}
// `curve surface op ...`: the IntCurveSurface_InterUtils point functions: 0 ComputeTransitions
// (w u v), 1 ComputeParamsOnQuadric (p u v), 2 ComputeAppendPoint (w u v), 3 ProcessIntAna of
// the line, 4 PerformCurveQuadric, 5 ProcessLinTorus, 6 SortStartPoints (n (u v w)...),
// 7 ClampUVParameters (4), 8 DecomposeSurfaceIntervals, 9 DoSurface and DoNewBounds (u0 u1
// v0 v1 x(2) y(2) z(2)), 10 SectionPointToParameters (polygon polyhedron n sections),
// 11 Collect, Sort and ProcessSortedPoints (polygon polyhedron u0 u1 v0 v1).
void ga_ics_interutils_case(std::istream& input){
 GaReader r{input};occ::handle<Adaptor3d_Curve> c=ga_ics_curve(r);occ::handle<Adaptor3d_Surface> s=ga_ics_surface(r);int op=r.integer();
 namespace IU=IntCurveSurface_InterUtils;using C=occ::handle<Adaptor3d_Curve>;using S=occ::handle<Adaptor3d_Surface>;
 using CT=IntCurveSurface_TheHCurveTool;using ST=Adaptor3d_HSurfaceTool;NCollection_DynamicArray<IntCurveSurface_IntersectionPoint> pts;
 GA_TRY(if(op==0){double w=r.real(),u=r.real(),v=r.real();IntCurveSurface_TransitionOnCurve t=IntCurveSurface_Out;
   IU::ComputeTransitions<C,CT,S,ST>(c,w,t,s,u,v);ga_int(t);}
  else if(op==1){gp_Pnt p=r.pnt();double u=r.real(),v=r.real();IU::ComputeParamsOnQuadric<S,ST>(s,p,u,v);ga_real(u);ga_real(v);}
  else if(op==2){double w=r.real(),u=r.real(),v=r.real();IntCurveSurface_IntersectionPoint p;ga_flag(IU::ComputeAppendPoint<C,CT,S,ST>(c,w,s,u,v,p));ga_ics_ipoint(p);}
  else if(op==3){bool par=false;ga_flag(IU::ProcessIntAna<C,CT,S,ST>(c,s,ga_ics_line_quadric(c,s),par,pts));ga_flag(par);ga_ics_points_print(pts);}
  else if(op==4){IU::PerformCurveQuadric<IntCurveSurface_TheQuadCurvExactHInter,C,CT,S,ST>(c,s,pts);ga_ics_points_print(pts);}
  else if(op==5){ga_flag(IU::ProcessLinTorus<C,CT,S,ST>(c->Line(),c,s,pts));ga_ics_points_print(pts);}
  else if(op==6){IU::SortedStartPoints sp;int n=r.integer();for(int i=0;i<n;++i){double u=r.real(),v=r.real();sp.Append(u,v,r.real());}
   IU::SortStartPoints(sp);ga_ics_start_print(sp);}
  else if(op==7){double b[4];for(double& x:b)x=r.real();IU::ClampUVParameters(b[0],b[1],b[2],b[3]);for(double x:b)ga_real(x);}
  else if(op==8){NCollection_DynamicArray<IU::UVBounds> iv;IU::DecomposeSurfaceIntervals<S,ST>(s,iv);ga_int(int(iv.Length()));
   for(const IU::UVBounds& b:iv){ga_real(b.U0);ga_real(b.U1);ga_real(b.V0);ga_real(b.V1);}}
  else if(op==9)ga_ics_sampling(r,s);
  else if(op==10){IntCurveSurface_ThePolygonOfHInter pg=ga_ics_polygon(r,c);IntCurveSurface_ThePolyhedronOfHInter ph=ga_ics_polyhedron(r,s);
   int n=r.integer();for(int i=0;i<n;++i){Intf_SectionPoint sp=ga_spoint(r);GA_TRY(double u,v,w;IU::SectionPointToParameters(sp,ph,pg,u,v,w);ga_real(u);ga_real(v);ga_real(w));}}
  else ga_ics_refine(r,c,s));
 ga_end();
}
