// BRepIntCurveSurface probe operations (docs/topalgo-design.md section 12): the
// intersections of lines and curves with shapes read by tp_shape
// (tktopalgo_probe_shape.cxx). Faces are printed as their index among the faces of the
// shape (TopExp_Explorer order), states and transitions as integers. The Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_int_curve_surface.rs. Needs tp_bra_bspline
// (tktopalgo_probe_brepapprox.cxx).
#include <BRepIntCurveSurface_Inter.hxx>
#include <Geom_Circle.hxx>
#include <Geom_Parabola.hxx>
#include <IntCurveSurface_IntersectionPoint.hxx>
// The 1-based index of `face` among the faces of `shape`, -1 when it is not one of them.
int tp_bics_face_index(const TopoDS_Shape& shape,const TopoDS_Face& face){
 int i=0;for(TopExp_Explorer e(shape,TopAbs_FACE);e.More();e.Next()){++i;if(e.Current().IsSame(face))return i;}return -1;}
// Every accessor of the current point.
void tp_bics_current(const TopoDS_Shape& shape,const BRepIntCurveSurface_Inter& s){
 GA_TRY(ga_int(tp_bics_face_index(shape,s.Face())));
 GA_TRY(IntCurveSurface_IntersectionPoint p=s.Point();ga_pnt(p.Pnt());ga_real(p.U());ga_real(p.V());ga_real(p.W());ga_int(int(p.Transition())));
 GA_TRY(ga_pnt(s.Pnt()));GA_TRY(ga_real(s.U()));GA_TRY(ga_real(s.V()));GA_TRY(ga_real(s.W()));
 GA_TRY(ga_int(int(s.State())));GA_TRY(ga_int(int(s.Transition())));}
// The points of the exploration (at most 64), then the accessors past the end, one more
// Next and More.
void tp_bics_walk(const TopoDS_Shape& shape,BRepIntCurveSurface_Inter& s){
 int n=0;for(;s.More()&&n<64;++n){tp_bics_current(shape,s);GA_TRY(s.Next());}
 ga_int(n);ga_flag(s.More());tp_bics_current(shape,s);GA_TRY(s.Next());ga_flag(s.More());GA_TRY(ga_int(tp_bics_face_index(shape,s.Face())));}
// `kind fields first last`: a GeomAdaptor_Curve on 0 the line P D, 1 the circle P N X R,
// 2 a B-spline curve (tp_bra_bspline), 3 the parabola P N X focal.
occ::handle<GeomAdaptor_Curve> tp_bics_curve(GaReader& r){
 int kind=r.integer();occ::handle<Geom_Curve> c;
 if(kind==0){gp_Pnt p=r.pnt();c=new Geom_Line(p,r.dir());}
 else if(kind==2)c=tp_bra_bspline(r);
 else{gp_Pnt p=r.pnt();gp_Dir n=r.dir(),x=r.dir();gp_Ax2 a(p,n,x);double v=r.real();
  if(kind==1)c=new Geom_Circle(a,v);else c=new Geom_Parabola(a,v);}
 double first=r.real(),last=r.real();return new GeomAdaptor_Curve(c,first,last);}
// `shape tol n (P D)...`: Init(shape, line, tol) on one object for each line, then the
// exploration.
void tp_bics_line_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape sh=tp_shape(r);double tol=r.real();int n=r.integer();BRepIntCurveSurface_Inter s;
  for(int k=0;k<n;++k){gp_Pnt p=r.pnt();gp_Dir d=r.dir();GA_TRY(s.Init(sh.shape,gp_Lin(p,d),tol));tp_bics_walk(sh.shape,s);});
 ga_end();}
// `shape tol mode reload n (curve)...`: mode 0 Load (twice with reload) then Init(curve)
// for each curve, mode 1 Init(shape, curve, tol) for each curve.
void tp_bics_curve_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape sh=tp_shape(r);double tol=r.real();int mode=r.integer();bool reload=r.flag();int n=r.integer();BRepIntCurveSurface_Inter s;
  if(mode==0){s.Load(sh.shape,tol);if(reload)s.Load(sh.shape,tol);ga_flag(s.More());}
  for(int k=0;k<n;++k){occ::handle<GeomAdaptor_Curve> c=tp_bics_curve(r);
   if(mode==0){GA_TRY(s.Init(*c));}else{GA_TRY(s.Init(sh.shape,*c,tol));}tp_bics_walk(sh.shape,s);});
 ga_end();}
// `shape tol`: a fresh object (More and the accessors), Init of a line segment before any
// Load, then Load of the shape (More, Face, the accessors, then Next and the exploration
// with the segment) and Load of an empty compound followed by two Next.
void tp_bics_state_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape sh=tp_shape(r);double tol=r.real();BRepIntCurveSurface_Inter s;ga_flag(s.More());tp_bics_current(sh.shape,s);
  occ::handle<Geom_Curve> l=new Geom_Line(gp_Pnt(0,0,0),gp_Dir(0,0,1));GA_TRY(s.Init(GeomAdaptor_Curve(l,-1.0,1.0)));ga_flag(s.More());
  s.Load(sh.shape,tol);ga_flag(s.More());tp_bics_current(sh.shape,s);GA_TRY(s.Next());tp_bics_walk(sh.shape,s);
  TopoDS_Compound empty;BRep_Builder().MakeCompound(empty);s.Load(empty,tol);ga_flag(s.More());
  GA_TRY(s.Next());ga_flag(s.More());GA_TRY(s.Next());ga_flag(s.More());tp_bics_current(sh.shape,s));
 ga_end();}
