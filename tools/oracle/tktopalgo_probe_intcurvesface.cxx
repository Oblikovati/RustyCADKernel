// IntCurvesFace probe operations over faces read by tp_face (tktopalgo_probe_face.cxx);
// the Rust twin is crates/tktopalgo/examples/tp_probe/int_curves_face.rs. States and
// transitions are printed as integers, faces of a shape as their index in the shape.
#include <Geom_BezierCurve.hxx>
#include <Geom_Circle.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <IntCurvesFace_Intersector.hxx>
#include <IntCurvesFace_ShapeIntersector.hxx>
#include <TopoDS_Compound.hxx>
#include <gp_Lin.hxx>
// The point accessors of an intersector (or a shape intersector) for points 1..NbPnt,
// then at 0 and NbPnt + 1.
template<class C> void tp_icf_point(const C& c,int i){
 GA_TRY(ga_real(c.UParameter(i));ga_real(c.VParameter(i));ga_real(c.WParameter(i));
  ga_pnt(c.Pnt(i));ga_int(c.Transition(i));ga_int(c.State(i)))}
template<class C> void tp_icf_points(const C& c){
 ga_flag(c.IsDone());ga_int(c.NbPnt());
 for(int i=1;i<=c.NbPnt();++i)tp_icf_point(c,i);
 tp_icf_point(c,0);tp_icf_point(c,c.NbPnt()+1);}
// The results of one Perform: points, parallel flag and the classification of each UV.
void tp_icf_results(const IntCurvesFace_Intersector& c){
 tp_icf_points(c);ga_flag(c.IsParallel());
 for(int i=1;i<=c.NbPnt();++i)GA_TRY(ga_int(c.ClassifyUVPoint(gp_Pnt2d(c.UParameter(i),c.VParameter(i)))))}
// `face tol restr usebtol n (P D pmin pmax)...`: one intersector, its surface type and
// bounding box, then the results of each line in turn.
void tp_icf_line_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);double tol=r.real();bool restr=r.flag(),usebtol=r.flag();
  occ::handle<IntCurvesFace_Intersector> c=new IntCurvesFace_Intersector(f.face,tol,restr,usebtol);
  ga_int(c->SurfaceType());ga_flag(c->GetUseBoundToler());ga_flag(c->IsDone());
  Bnd_Box b=c->Bounding();ga_flag(b.IsVoid());
  if(!b.IsVoid()){double x0,y0,z0,x1,y1,z1;b.Get(x0,y0,z0,x1,y1,z1);ga_real(x0);ga_real(y0);ga_real(z0);ga_real(x1);ga_real(y1);ga_real(z1);}
  int n=r.integer();
  for(int i=0;i<n;++i){gp_Pnt p=r.pnt();gp_Dir d=r.dir();double a=r.real(),e=r.real();
   GA_TRY(c->Perform(gp_Lin(p,d),a,e);tp_icf_results(*c))}
  c->SetUseBoundToler(!usebtol);ga_flag(c->GetUseBoundToler());ga_flag(c->Face().IsSame(f.face)))
 ga_end();}
// `kind fields... first last`: a 3d curve adaptor: 0 line P D, 1 circle P N R, 2 Bezier
// curve n (x y z)...
occ::handle<Adaptor3d_Curve> tp_icf_curve(GaReader& r){
 int kind=r.integer();occ::handle<Geom_Curve> c;
 if(kind==0){gp_Pnt p=r.pnt();c=new Geom_Line(p,r.dir());}
 else if(kind==1){gp_Pnt p=r.pnt();gp_Dir n=r.dir();c=new Geom_Circle(gp_Ax2(p,n),r.real());}
 else{int n=r.integer();NCollection_Array1<gp_Pnt> poles(1,n);for(int i=1;i<=n;++i)poles(i)=r.pnt();c=new Geom_BezierCurve(poles);}
 double first=r.real(),last=r.real();return new GeomAdaptor_Curve(c,first,last);}
// `face tol restr usebtol curve pmin pmax`: the results of Perform on a curve adaptor.
void tp_icf_curve_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFace f=tp_face(r);double tol=r.real();bool restr=r.flag(),usebtol=r.flag();
  occ::handle<Adaptor3d_Curve> hc=tp_icf_curve(r);double a=r.real(),e=r.real();
  IntCurvesFace_Intersector c(f.face,tol,restr,usebtol);c.Perform(hc,a,e);tp_icf_results(c))
 ga_end();}
// The index of the face of the i-th point among the faces of the compound.
int tp_icf_face_index(const std::vector<TpFace>& faces,const TopoDS_Face& face){
 for(size_t k=0;k<faces.size();++k)if(faces[k].face.IsSame(face))return int(k);return -1;}
// `nfaces (face)... tol reload mode n (P D pmin pmax)...`: a compound of the faces loaded
// (twice with reload) in a shape intersector, then n line requests: mode 0 Perform, 1
// PerformNearest, 2 Perform of a GeomAdaptor_Curve on the line over [pmin, pmax].
void tp_icf_shape_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int nf=r.integer();std::vector<TpFace> faces;BRep_Builder B;TopoDS_Compound comp;B.MakeCompound(comp);
  for(int k=0;k<nf;++k){faces.push_back(tp_face(r));B.Add(comp,faces.back().face);}
  double tol=r.real();bool reload=r.flag();int mode=r.integer();
  IntCurvesFace_ShapeIntersector s;ga_flag(s.IsDone());s.Load(comp,tol);if(reload)s.Load(comp,tol);
  int n=r.integer();
  for(int i=0;i<n;++i){gp_Pnt p=r.pnt();gp_Dir d=r.dir();double a=r.real(),e=r.real();
   GA_TRY(if(mode==0)s.Perform(gp_Lin(p,d),a,e);else if(mode==1)s.PerformNearest(gp_Lin(p,d),a,e);
    else s.Perform(occ::handle<Adaptor3d_Curve>(new GeomAdaptor_Curve(new Geom_Line(p,d),a,e)),a,e);
    tp_icf_points(s);for(int j=1;j<=s.NbPnt();++j)ga_int(tp_icf_face_index(faces,s.Face(j)));
    GA_TRY(s.Face(0))GA_TRY(s.Face(s.NbPnt()+1)))}
  s.SortResult();ga_flag(s.IsDone());ga_int(s.NbPnt()))
 ga_end();}
