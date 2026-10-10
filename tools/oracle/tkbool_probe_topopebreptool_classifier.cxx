// TopOpeBRepTool_SolidClassifier, TopOpeBRepTool_ShapeClassifier, the FSC_* functions
// and TOOL::TggeomE probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/top_ope_brep_tool_classifier.rs.
//
// The shapes are shapes of a shape program (tktopalgo_probe_shapes.cxx); states print as
// TopAbs_State values.
#include <TopOpeBRepTool_SC.hxx>
#include <TopOpeBRepTool_ShapeClassifier.hxx>
#include <TopOpeBRepTool_SolidClassifier.hxx>
#include <TopOpeBRepTool_TOOL.hxx>
// `program shell solid tol n (x y z)...`: per point Classify on the shell, then on the
// solid, and State(), with one classifier kept over the points.
void bl_tt_solid_classifier_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shell sh=TopoDS::Shell(t.shapes.at(r.integer()));const TopoDS_Solid so=TopoDS::Solid(t.shapes.at(r.integer()));
 double tol=r.real();int n=r.integer();
 TopOpeBRepTool_SolidClassifier sc;
 for(int i=0;i<n;i++){gp_Pnt p=r.pnt();
  GA_TRY(ga_int(sc.Classify(sh,p,tol)))
  GA_TRY(ga_int(sc.Classify(so,p,tol)))
  ga_int(sc.State());}
 ga_end();}
// The state of the classifier, then its 3d and 2d points (raising when undefined).
void bl_tt_sc_result(const TopOpeBRepTool_ShapeClassifier& sc,TopAbs_State st){
 ga_int(st);ga_int(sc.State());
 GA_TRY(ga_pnt(sc.P3D()))
 GA_TRY(ga_pnt2d(sc.P2D()))}
// `program s ref samedomain avoid`: StateShapeShape(s, ref, samedomain), then (avoid >= 0)
// avoiding the shape and avoiding the list of it, then SetReference(ref) and
// StateShapeReference(s, avoid or null); each prints its state and points.
void bl_tt_shape_classifier_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape s=t.shapes.at(r.integer()),ref=t.shapes.at(r.integer());int sd=r.integer(),av=r.integer();
 const TopoDS_Shape avs=av>=0?t.shapes.at(av):TopoDS_Shape();
 TopOpeBRepTool_ShapeClassifier sc;
 GA_TRY(TopAbs_State st=sc.StateShapeShape(s,ref,sd);bl_tt_sc_result(sc,st))
 if(av>=0){
  GA_TRY(TopAbs_State st=sc.StateShapeShape(s,avs,ref);bl_tt_sc_result(sc,st))
  GA_TRY(NCollection_List<TopoDS_Shape> l;l.Append(avs);TopAbs_State st=sc.StateShapeShape(s,l,ref);bl_tt_sc_result(sc,st))}
 GA_TRY(sc.SameDomain(sd);sc.SetReference(ref);TopAbs_State st=sc.StateShapeReference(s,avs);bl_tt_sc_result(sc,st))
 ga_end();}
// `program e f t x y z`: FSC_StateEonFace(e, t, f) and FSC_StatePonFace(p, f) with the
// per-thread classifier, then FSC_GetPSC(f) and StateShapeReference(e, null).
void bl_tt_fsc_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Shape e=t.shapes.at(r.integer()),f=t.shapes.at(r.integer());double tt=r.real();gp_Pnt p=r.pnt();
 GA_TRY(ga_int(FSC_StateEonFace(e,tt,f,FSC_GetPSC())))
 GA_TRY(ga_int(FSC_StatePonFace(p,f,FSC_GetPSC())))
 GA_TRY(TopOpeBRepTool_ShapeClassifier& sc=FSC_GetPSC(f);ga_int(sc.StateShapeReference(e,TopoDS_Shape())))
 ga_end();}
// `program e n par...`: TOOL::TggeomE(par, E) of each parameter (flag and tangent).
void bl_tt_tggeom_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Edge e=TopoDS::Edge(t.shapes.at(r.integer()));int n=r.integer();
 for(int i=0;i<n;i++){double par=r.real();
  GA_TRY(gp_Vec tg(0,0,0);bool ok=TopOpeBRepTool_TOOL::TggeomE(par,e,tg);ga_flag(ok);ga_xyz(tg.XYZ()))}
 ga_end();}
