// TopOpeBRepTool_ShapeTool, PROJECT, GEOMETRY, 2d (FC2D), tol (FTOL), HBoxTool and
// TOOL::UVISO probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/top_ope_brep_tool_shapes.rs.
//
// The shapes are shapes of a shape program (tktopalgo_probe_shapes.cxx); curves print
// through bl_curve2d (tkbool_probe_brepfill_multi_line.cxx).
#include <BRepAdaptor_Curve2d.hxx>
#include <TopOpeBRepTool_2d.hxx>
#include <TopOpeBRepTool_GEOMETRY.hxx>
#include <TopOpeBRepTool_HBoxTool.hxx>
#include <TopOpeBRepTool_PROJECT.hxx>
#include <TopOpeBRepTool_ShapeTool.hxx>
#include <TopOpeBRepTool_TOOL.hxx>
#include <TopOpeBRepTool_tol.hxx>
// `program s1 s2 e f par u v`: the ShapeTool members on the shapes, the edge and the
// face: Tolerance, Pnt, ShapesSameOriented, the basis surface type, UVBOUNDS,
// AdjustOnPeriodic, Closed, PeriodizeParameter, EdgeData and Resolution3d.
void bl_tt_shape_tool_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape s1=t.shapes.at(r.integer()),s2=t.shapes.at(r.integer()),e=t.shapes.at(r.integer()),f=t.shapes.at(r.integer());
 double par=r.real(),u=r.real(),v=r.real();
 GA_TRY(ga_real(TopOpeBRepTool_ShapeTool::Tolerance(s1)))
 GA_TRY(ga_pnt(TopOpeBRepTool_ShapeTool::Pnt(s1)))
 GA_TRY(ga_flag(TopOpeBRepTool_ShapeTool::ShapesSameOriented(s1,s2)))
 GA_TRY(ga_text(TopOpeBRepTool_ShapeTool::BASISSURFACE(TopoDS::Face(f))->DynamicType()->Name()))
 GA_TRY(bool up=false,vp=false;double a=-1,b=-1,c=-1,d=-1;TopOpeBRepTool_ShapeTool::UVBOUNDS(TopoDS::Face(f),up,vp,a,b,c,d);
  ga_flag(up);ga_flag(vp);ga_real(a);ga_real(b);ga_real(c);ga_real(d))
 GA_TRY(double uu=u,vv=v;TopOpeBRepTool_ShapeTool::AdjustOnPeriodic(f,uu,vv);ga_real(uu);ga_real(vv))
 GA_TRY(ga_flag(TopOpeBRepTool_ShapeTool::Closed(e,f)))
 GA_TRY(ga_real(TopOpeBRepTool_ShapeTool::PeriodizeParameter(par,e,f)))
 GA_TRY(gp_Dir T(gp_Dir::D::Z),N(gp_Dir::D::Z);double C=-1;ga_real(TopOpeBRepTool_ShapeTool::EdgeData(e,par,T,N,C));
  ga_dir(T);ga_dir(N);ga_real(C))
 GA_TRY(ga_real(TopOpeBRepTool_ShapeTool::Resolution3d(TopoDS::Face(f),1.e-9)))
 ga_end();}
// `program e f x y z`: the PROJECT functions of the point on the edge and the face:
// FUN_tool_bounds, projPonE, projPonF, projPonboundedF and projPonC2D (on the p-curve
// of the edge on the face).
void bl_tt_project_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Edge e=TopoDS::Edge(t.shapes.at(r.integer()));const TopoDS_Face f=TopoDS::Face(t.shapes.at(r.integer()));
 gp_Pnt p=r.pnt();
 GA_TRY(double a,b;FUN_tool_bounds(e,a,b);ga_real(a);ga_real(b))
 GA_TRY(double pa=-1,d=-1;ga_flag(FUN_tool_projPonE(p,e,pa,d));ga_real(pa);ga_real(d))
 GA_TRY(gp_Pnt2d uv(-1,-1);double d=-1;ga_flag(FUN_tool_projPonF(p,f,uv,d));ga_pnt2d(uv);ga_real(d))
 GA_TRY(gp_Pnt2d uv(-1,-1);double d=-1;ga_flag(FUN_tool_projPonboundedF(p,f,uv,d));ga_pnt2d(uv);ga_real(d))
 GA_TRY(BRepAdaptor_Curve2d bc(e,f);double pa=-1,d=-1;ga_flag(FUN_tool_projPonC2D(p,bc,pa,d));ga_real(pa);ga_real(d))
 ga_end();}
// `program e f u v`: the GEOMETRY functions on the curve of the edge at the middle of
// its range and on the surface of the face at (u, v): dirC, line, quad, ngS, onapex,
// quad and closed.
void bl_tt_geometry_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Edge e=TopoDS::Edge(t.shapes.at(r.integer()));const TopoDS_Face f=TopoDS::Face(t.shapes.at(r.integer()));
 gp_Pnt2d uv=r.pnt2d();
 double a,b;occ::handle<Geom_Curve> c=BRep_Tool::Curve(e,a,b);occ::handle<Geom_Surface> s=BRep_Tool::Surface(f);
 GA_TRY(ga_dir(FUN_tool_dirC(0.5*(a+b),c)))
 GA_TRY(ga_flag(FUN_tool_line(c)))
 GA_TRY(ga_flag(FUN_tool_quad(c)))
 GA_TRY(ga_dir(FUN_tool_ngS(uv,s)))
 GA_TRY(ga_flag(FUN_tool_onapex(uv,s)))
 GA_TRY(ga_flag(FUN_tool_quad(s)))
 GA_TRY(bool uc=false,vc=false;double up=-1,vp=-1;ga_flag(FUN_tool_closed(s,uc,up,vc,vp));ga_flag(uc);ga_real(up);ga_flag(vc);ga_real(vp))
 ga_end();}
// `program s1 s2 e f ef`: FC2D_Prepare(s1, s2), then the p-curve functions of the edge
// on the face: HasC3D, HasOld, CurveOnSurface, HasNew, EditableCurveOnSurface,
// TOOL::UVISO, CurveOnSurface translated onto `ef` and FTOL_FaceTolerances3d(f, f).
void bl_tt_fc2d_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape s1=t.shapes.at(r.integer()),s2=t.shapes.at(r.integer());
 const TopoDS_Edge e=TopoDS::Edge(t.shapes.at(r.integer()));const TopoDS_Face f=TopoDS::Face(t.shapes.at(r.integer()));
 const TopoDS_Edge ef=TopoDS::Edge(t.shapes.at(r.integer()));
 FC2D_Prepare(s1,s2);
 GA_TRY(ga_flag(FC2D_HasC3D(e)))
 GA_TRY(occ::handle<Geom2d_Curve> c;ga_flag(FC2D_HasOldCurveOnSurface(e,f,c)))
 GA_TRY(double a=-1,b=-1,tol=-1;occ::handle<Geom2d_Curve> c=FC2D_CurveOnSurface(e,f,a,b,tol);bl_curve2d(c);ga_real(a);ga_real(b);ga_real(tol))
 GA_TRY(occ::handle<Geom2d_Curve> c;ga_flag(FC2D_HasNewCurveOnSurface(e,f,c)))
 GA_TRY(double a=-1,b=-1,tol=-1;occ::handle<Geom2d_Curve> c=FC2D_EditableCurveOnSurface(e,f,a,b,tol);bl_curve2d(c);ga_real(a);ga_real(b);ga_real(tol))
 GA_TRY(bool iu=false,iv=false;gp_Dir2d d(gp_Dir2d::D::X);gp_Pnt2d o(0,0);ga_flag(TopOpeBRepTool_TOOL::UVISO(e,f,iu,iv,d,o));
  ga_flag(iu);ga_flag(iv);ga_xy(d.XY());ga_pnt2d(o))
 GA_TRY(double a=-1,b=-1,tol=-1;occ::handle<Geom2d_Curve> c=FC2D_CurveOnSurface(e,f,ef,a,b,tol);bl_curve2d(c);ga_real(a);ga_real(b);ga_real(tol))
 GA_TRY(double tol=-1;FTOL_FaceTolerances3d(f,f,tol);ga_real(tol))
 ga_end();}
// `program s ts`: a TopOpeBRepTool_HBoxTool with the boxes of the sub-shapes of type
// `ts`; prints Extent, each box, the index of each shape, and the box on the vertices.
void bl_tt_hbox_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape s=t.shapes.at(r.integer());int ts=r.integer();
 occ::handle<TopOpeBRepTool_HBoxTool> hbt=new TopOpeBRepTool_HBoxTool();
 GA_TRY(hbt->AddBoxes(s,(TopAbs_ShapeEnum)ts))
 ga_int(hbt->Extent());
 for(int i=1;i<=hbt->Extent();i++){
  GA_TRY(double x0,y0,z0,x1,y1,z1;hbt->Box(i).Get(x0,y0,z0,x1,y1,z1);ga_real(x0);ga_real(y0);ga_real(z0);ga_real(x1);ga_real(y1);ga_real(z1))
  ga_int(hbt->Index(hbt->Shape(i)));}
 GA_TRY(hbt->Box(hbt->Extent()+1))
 GA_TRY(Bnd_Box b;TopOpeBRepTool_HBoxTool::ComputeBoxOnVertices(s,b);double x0,y0,z0,x1,y1,z1;b.Get(x0,y0,z0,x1,y1,z1);
  ga_real(x0);ga_real(y0);ga_real(z0);ga_real(x1);ga_real(y1);ga_real(z1))
 ga_end();}
