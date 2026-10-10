// BRepFill_MultiLine, BRepFill_ApproxSeewing and BRepFill_TrimSurfaceTool probe
// operations; the Rust twin is crates/tkbool/examples/bl_probe/brep_fill_multi_line.rs.
//
// The faces and edges are shapes of a shape program and the bisector one of its
// p-curves (tktopalgo_probe_shapes.cxx).
#include <BRepFill_ApproxSeewing.hxx>
#include <BRepFill_MultiLine.hxx>
#include <BRepFill_TrimSurfaceTool.hxx>
// The range and the points at the ends and the middle of a curve (or `-1` for null).
static void bl_curve3d(const occ::handle<Geom_Curve>& c){
 if(c.IsNull()){ga_int(-1);return;}
 double f=c->FirstParameter(),l=c->LastParameter();ga_real(f);ga_real(l);
 for(double u:{f,0.5*(f+l),l})ga_pnt(c->Value(u));}
static void bl_curve2d(const occ::handle<Geom2d_Curve>& c){
 if(c.IsNull()){ga_int(-1);return;}
 double f=c->FirstParameter(),l=c->LastParameter();ga_real(f);ga_real(l);
 for(double u:{f,0.5*(f+l),l})ga_pnt2d(c->Value(u));}
// `program f1 f2 e1 e2 inv1 inv2 bis n`: a BRepFill_MultiLine on the faces and edges
// along the p-curve `bis`; prints IsParticularCase, Continuity, the range, the points on
// both faces and in 3d at n parameters, the curves of the particular cases and the
// BRepFill_ApproxSeewing of the line.
void bl_bf_multi_line_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Face f1=TopoDS::Face(t.shapes.at(r.integer())),f2=TopoDS::Face(t.shapes.at(r.integer()));
 const TopoDS_Edge e1=TopoDS::Edge(t.shapes.at(r.integer())),e2=TopoDS::Edge(t.shapes.at(r.integer()));
 bool inv1=r.flag(),inv2=r.flag();const occ::handle<Geom2d_Curve> bis=t.pcurves.at(r.integer());int n=r.integer();
 std::unique_ptr<BRepFill_MultiLine> ml;
 try{ml.reset(new BRepFill_MultiLine(f1,f2,e1,e2,inv1,inv2,bis));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 ga_flag(ml->IsParticularCase());ga_int((int)ml->Continuity());
 double u1=ml->FirstParameter(),u2=ml->LastParameter();ga_real(u1);ga_real(u2);
 for(int i=0;i<n;i++){double u=u1+i*(u2-u1)/(n-1);
  GA_TRY(ga_pnt2d(ml->ValueOnF1(u));ga_pnt2d(ml->ValueOnF2(u));ga_pnt(ml->Value(u)))}
 GA_TRY(occ::handle<Geom_Curve> c;occ::handle<Geom2d_Curve> p1,p2;ml->Curves(c,p1,p2);bl_curve3d(c);bl_curve2d(p1);bl_curve2d(p2))
 GA_TRY(BRepFill_ApproxSeewing app(*ml);ga_flag(app.IsDone());bl_curve3d(app.Curve());bl_curve2d(app.CurveOnF1());bl_curve2d(app.CurveOnF2()))
 ga_end();}
// `program bis f1 f2 e1 e2 inv1 inv2 ef1 ef2 x y pe u1 u2`: a BRepFill_TrimSurfaceTool;
// prints IntersectWith(ef1, ef2), IsOnFace(x, y), ProjOn((x, y), pe) and Project(u1, u2)
// (its curves and continuity).
void bl_bf_trim_surface_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const occ::handle<Geom2d_Curve> bis=t.pcurves.at(r.integer());
 const TopoDS_Face f1=TopoDS::Face(t.shapes.at(r.integer())),f2=TopoDS::Face(t.shapes.at(r.integer()));
 const TopoDS_Edge e1=TopoDS::Edge(t.shapes.at(r.integer())),e2=TopoDS::Edge(t.shapes.at(r.integer()));
 bool inv1=r.flag(),inv2=r.flag();
 const TopoDS_Edge ef1=TopoDS::Edge(t.shapes.at(r.integer())),ef2=TopoDS::Edge(t.shapes.at(r.integer()));
 gp_Pnt2d p=r.pnt2d();const TopoDS_Edge pe=TopoDS::Edge(t.shapes.at(r.integer()));double u1=r.real(),u2=r.real();
 BRepFill_TrimSurfaceTool tool(bis,f1,f2,e1,e2,inv1,inv2);
 GA_TRY(NCollection_Sequence<gp_Pnt> pts;tool.IntersectWith(ef1,ef2,pts);ga_int(pts.Length());for(const gp_Pnt& q:pts)ga_pnt(q))
 GA_TRY(ga_flag(tool.IsOnFace(p)))
 GA_TRY(ga_real(tool.ProjOn(p,pe)))
 GA_TRY(occ::handle<Geom_Curve> c;occ::handle<Geom2d_Curve> p1,p2;GeomAbs_Shape cont;tool.Project(u1,u2,c,p1,p2,cont);
  ga_int((int)cont);bl_curve3d(c);bl_curve2d(p1);bl_curve2d(p2))
 ga_end();}
