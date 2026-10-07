// TopOpeBRepTool_GeomTool and TopOpeBRepTool_CurveTool probe operations; the Rust twin
// is crates/tkbool/examples/bl_probe/top_ope_brep_tool_curve_tool.rs.
//
// The faces and curves are shapes and curves of a shape program
// (tktopalgo_probe_shapes.cxx); curves print through bl_curve3d and bl_curve2d
// (tkbool_probe_brepfill_multi_line.cxx).
#include <TopOpeBRepTool_CurveTool.hxx>
#include <TopOpeBRepTool_GeomTool.hxx>
// The poles count and the curve of a B-spline (or `-1` for another curve).
static void bl_poles3d(const occ::handle<Geom_Curve>& c){
 occ::handle<Geom_BSplineCurve> b=occ::down_cast<Geom_BSplineCurve>(c);
 if(b.IsNull()){ga_int(-1);return;}ga_int(b->NbPoles());ga_int(b->Degree());bl_curve3d(c);}
static void bl_poles2d(const occ::handle<Geom2d_Curve>& c){
 occ::handle<Geom2d_BSplineCurve> b=occ::down_cast<Geom2d_BSplineCurve>(c);
 if(b.IsNull()){ga_int(-1);return;}ga_int(b->NbPoles());ga_int(b->Degree());bl_curve2d(c);}
// `program f1 f2 c3d pc1 pc2 tol3d tol2d nbpntmax parmin parmax n (x y z u1 v1 u2 v2)...`:
// a TopOpeBRepTool_CurveTool(APPROX) with the curves asked by the flags and the
// tolerances set; MakeCurves of the B-splines of degree 1 (MakeBSpline1fromPnt(2d))
// through the points; prints the result, the new curves and the reached tolerances,
// then the input curves (replaced natively when they have more than 100 poles).
void bl_tt_make_curves_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape f1=t.shapes.at(r.integer()),f2=t.shapes.at(r.integer());
 bool c3d=r.flag(),pc1=r.flag(),pc2=r.flag();double tol3d=r.real(),tol2d=r.real();int nbpntmax=r.integer();
 double parmin=r.real(),parmax=r.real();int n=r.integer();
 NCollection_Array1<gp_Pnt> p(1,n);NCollection_Array1<gp_Pnt2d> p1(1,n),p2(1,n);
 for(int i=1;i<=n;i++){p(i)=r.pnt();p1(i)=r.pnt2d();p2(i)=r.pnt2d();}
 TopOpeBRepTool_CurveTool ct(TopOpeBRepTool_APPROX);
 ct.ChangeGeomTool().Define(TopOpeBRepTool_APPROX,c3d,pc1,pc2);
 ct.ChangeGeomTool().SetTolerances(tol3d,tol2d);ct.ChangeGeomTool().SetNbPntMax(nbpntmax);
 occ::handle<Geom_Curve> C3D=TopOpeBRepTool_CurveTool::MakeBSpline1fromPnt(p);
 occ::handle<Geom2d_Curve> PC1=TopOpeBRepTool_CurveTool::MakeBSpline1fromPnt2d(p1);
 occ::handle<Geom2d_Curve> PC2=TopOpeBRepTool_CurveTool::MakeBSpline1fromPnt2d(p2);
 occ::handle<Geom_Curve> C3Dnew;occ::handle<Geom2d_Curve> PC1new,PC2new;double t3=-1.,t2=-1.;
 try{ga_flag(ct.MakeCurves(parmin,parmax,C3D,PC1,PC2,f1,f2,C3Dnew,PC1new,PC2new,t3,t2));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 GA_TRY(bl_poles3d(C3Dnew);bl_poles2d(PC1new);bl_poles2d(PC2new))
 ga_real(t3);ga_real(t2);
 GA_TRY(bl_poles3d(C3D);bl_poles2d(PC1);bl_poles2d(PC2))
 ga_end();}
// `program face curve first last`: IsProjectable and MakePCurveOnFace of the program
// curve on the face; prints the flag, the p-curve and the reached 2d tolerance.
void bl_tt_pcurve_on_face_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape f=t.shapes.at(r.integer());const occ::handle<Geom_Curve> c=t.curves.at(r.integer());
 double first=r.real(),last=r.real();
 GA_TRY(ga_flag(TopOpeBRepTool_CurveTool::IsProjectable(f,c)))
 GA_TRY(double tol=-1.;occ::handle<Geom2d_Curve> pc=TopOpeBRepTool_CurveTool::MakePCurveOnFace(f,c,tol,first,last);
  bl_curve2d(pc);ga_real(tol))
 ga_end();}
