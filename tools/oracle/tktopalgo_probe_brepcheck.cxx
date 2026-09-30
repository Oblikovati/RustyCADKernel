// The BRepCheck (and BRepLib_ValidateEdge) operations of the TKTopAlgo oracle, over the
// shapes of tktopalgo_probe_shapes.cxx; the Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_check.rs.
//
// `bck_check kind program index script`: the checker of kind (0 vertex, 1 edge, 2 wire,
// 3 face, 4 shell, 5 solid) on shape `index`, then the script `n (code args)...`: the codes 90..99
// call the BRepCheck_Result members, 1.. the members of the checker (see
// tp_bck_member). After the script: the statuses of the shape and, in table order, of
// every shape with statuses. A context iteration prints its shapes sorted by index,
// since its order follows the TShape addresses.
#include <BRepCheck.hxx>
#include <BRepCheck_Edge.hxx>
#include <BRepCheck_Face.hxx>
#include <BRepCheck_Shell.hxx>
#include <BRepCheck_Solid.hxx>
#include <BRepCheck_Analyzer.hxx>
#include <BRepCheck_Vertex.hxx>
#include <BRepCheck_Wire.hxx>
#include <BRepLib_ValidateEdge.hxx>
#include <GeomAdaptor_Curve.hxx>
#include <GeomAdaptor_Surface.hxx>
#include <Geom2dAdaptor_Curve.hxx>
#include <Adaptor3d_CurveOnSurface.hxx>
#include <algorithm>
void tp_bck_list(const NCollection_List<BRepCheck_Status>& l){
 ga_int(l.Extent());for(BRepCheck_Status s:l)ga_int(int(s));}
// The context shapes of a checker, as sorted (index, statuses) pairs.
void tp_bck_contexts(const TpShapes& t,const occ::handle<BRepCheck_Result>& c){
 std::vector<std::pair<int,std::vector<int>>> found;
 for(c->InitContextIterator();c->MoreShapeInContext();c->NextShapeInContext()){
  std::vector<int> l;for(BRepCheck_Status s:c->StatusOnShape())l.push_back(int(s));
  found.emplace_back(tp_shape_index(t,c->ContextualShape()),l);}
 std::sort(found.begin(),found.end());ga_int(int(found.size()));
 for(auto& f:found){ga_int(f.first);ga_int(int(f.second.size()));for(int s:f.second)ga_int(s);}}
// The statuses of the checker's shape and of every table shape with statuses.
void tp_bck_summary(const TpShapes& t,const occ::handle<BRepCheck_Result>& c){
 GA_TRY(tp_bck_list(c->Status()));
 for(size_t i=0;i<t.shapes.size();++i)if(c->IsStatusOnShape(t.shapes[i])){ga_int(int(i));tp_bck_list(c->StatusOnShape(t.shapes[i]));}
 ga_int(-9);}
// The BRepCheck_Result members (codes 90..99); false for another code.
bool tp_bck_result_member(GaReader& r,const TpShapes& t,const occ::handle<BRepCheck_Result>& c,int code){
 switch(code){
 case 90:{int i=r.integer();GA_TRY(c->InContext(t.shapes.at(i));ga_int(0));return true;}
 case 91:c->Blind();return true;
 case 92:GA_TRY(tp_bck_list(c->Status()));return true;
 case 93:{int i=r.integer();GA_TRY(tp_bck_list(c->StatusOnShape(t.shapes.at(i))));return true;}
 case 94:ga_flag(c->IsStatusOnShape(t.shapes.at(r.integer())));return true;
 case 95:GA_TRY(tp_bck_contexts(t,c));return true;
 case 96:c->SetParallel(r.flag());ga_flag(c->IsParallel());return true;
 case 97:c->SetFailStatus(t.shapes.at(r.integer()));return true;
 case 98:{int i=r.integer();GA_TRY(c->Init(t.shapes.at(i));ga_int(0));return true;}
 case 99:ga_flag(c->IsMinimum());ga_flag(c->IsBlind());return true;}
 return false;}
void tp_bck_vertex_member(GaReader&,const TpShapes&,BRepCheck_Vertex& v,int code){
 if(code==1)GA_TRY(ga_real(v.Tolerance()));}
void tp_bck_edge_member(GaReader& r,const TpShapes& t,BRepCheck_Edge& e,int code){
 switch(code){
 case 1:GA_TRY(ga_real(e.Tolerance()));break;
 case 2:e.GeometricControls(r.flag());break;
 case 3:ga_flag(e.GeometricControls());break;
 case 4:e.SetExactMethod(r.flag());break;
 case 5:ga_flag(e.IsExactMethod());break;
 case 6:{int s=r.integer();GA_TRY(e.SetStatus(BRepCheck_Status(s)));break;}
 case 7:{int i=r.integer();GA_TRY(ga_int(int(e.CheckPolygonOnTriangulation(TopoDS::Edge(t.shapes.at(i))))));break;}}}
void tp_bck_wire_member(GaReader& r,const TpShapes& t,BRepCheck_Wire& w,int code){
 switch(code){
 case 1:{bool u=r.flag();GA_TRY(ga_int(int(w.Closed(u))));break;}
 case 2:{int f=r.integer();bool u=r.flag();GA_TRY(ga_int(int(w.Closed2d(TopoDS::Face(t.shapes.at(f)),u))));break;}
 case 3:{int f=r.integer();bool u=r.flag();GA_TRY(ga_int(int(w.Orientation(TopoDS::Face(t.shapes.at(f)),u))));break;}
 case 4:{int f=r.integer(),i1=r.integer(),i2=r.integer();bool u=r.flag();
  TopoDS_Edge e1=i1<0?TopoDS_Edge():TopoDS::Edge(t.shapes.at(i1)),e2=i2<0?TopoDS_Edge():TopoDS::Edge(t.shapes.at(i2));
  GA_TRY(ga_int(int(w.SelfIntersect(TopoDS::Face(t.shapes.at(f)),e1,e2,u))));ga_int(tp_shape_index(t,e1));ga_int(tp_shape_index(t,e2));break;}
 case 5:w.GeometricControls(r.flag());break;
 case 6:ga_flag(w.GeometricControls());break;
 case 7:{int s=r.integer();GA_TRY(w.SetStatus(BRepCheck_Status(s)));break;}}}
void tp_bck_face_member(GaReader& r,const TpShapes&,BRepCheck_Face& f,int code){
 switch(code){
 case 1:{bool u=r.flag();GA_TRY(ga_int(int(f.IntersectWires(u))));break;}
 case 2:{bool u=r.flag();GA_TRY(ga_int(int(f.ClassifyWires(u))));break;}
 case 3:{bool u=r.flag();GA_TRY(ga_int(int(f.OrientationOfWires(u))));break;}
 case 4:GA_TRY(f.SetUnorientable());break;
 case 5:GA_TRY(ga_flag(f.IsUnorientable()));break;
 case 6:{int s=r.integer();GA_TRY(f.SetStatus(BRepCheck_Status(s)));break;}
 case 7:f.GeometricControls(r.flag());break;
 case 8:ga_flag(f.GeometricControls());break;}}
// The sets of NbConnectedSet: the count, then each set's faces as sorted indices.
void tp_bck_sets(const TpShapes& t,BRepCheck_Shell& s){
 NCollection_List<TopoDS_Shape> sets;ga_int(s.NbConnectedSet(sets));
 for(const TopoDS_Shape& set:sets){std::vector<int> faces;ga_flag(set.Closed());
  for(TopExp_Explorer x(set,TopAbs_FACE);x.More();x.Next())faces.push_back(tp_same_index(t,x.Current()));
  std::sort(faces.begin(),faces.end());ga_int(int(faces.size()));for(int f:faces)ga_int(f);}}
void tp_bck_shell_member(GaReader& r,const TpShapes& t,BRepCheck_Shell& s,int code){
 switch(code){
 case 1:{bool u=r.flag();GA_TRY(ga_int(int(s.Closed(u))));break;}
 case 2:{bool u=r.flag();GA_TRY(ga_int(int(s.Orientation(u))));break;}
 case 3:GA_TRY(s.SetUnorientable());break;
 case 4:GA_TRY(ga_flag(s.IsUnorientable()));break;
 case 5:GA_TRY(tp_bck_sets(t,s));break;}}
occ::handle<BRepCheck_Result> tp_bck_checker(int kind,const TopoDS_Shape& s){
 if(kind==0)return new BRepCheck_Vertex(TopoDS::Vertex(s));
 if(kind==1)return new BRepCheck_Edge(TopoDS::Edge(s));
 if(kind==2)return new BRepCheck_Wire(TopoDS::Wire(s));
 if(kind==3)return new BRepCheck_Face(TopoDS::Face(s));
 if(kind==5)return new BRepCheck_Solid(TopoDS::Solid(s));
 return new BRepCheck_Shell(TopoDS::Shell(s));}
void tp_bck_member(GaReader& r,const TpShapes& t,const occ::handle<BRepCheck_Result>& c,int kind,int code){
 if(tp_bck_result_member(r,t,c,code))return;
 if(kind==0)tp_bck_vertex_member(r,t,*occ::down_cast<BRepCheck_Vertex>(c),code);
 else if(kind==1)tp_bck_edge_member(r,t,*occ::down_cast<BRepCheck_Edge>(c),code);
 else if(kind==2)tp_bck_wire_member(r,t,*occ::down_cast<BRepCheck_Wire>(c),code);
 else if(kind==3)tp_bck_face_member(r,t,*occ::down_cast<BRepCheck_Face>(c),code);
 else if(kind==4)tp_bck_shell_member(r,t,*occ::down_cast<BRepCheck_Shell>(c),code);}
void tp_bck_check_case(std::istream& in){
 GaReader r{in};int kind=r.integer();TpShapes t=tp_shapes(r);int index=r.integer();
 occ::handle<BRepCheck_Result> c;
 try{c=tp_bck_checker(kind,t.shapes.at(index));}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 int n=r.integer();
 for(int k=0;k<n;++k)tp_bck_member(r,t,c,kind,r.integer());
 tp_bck_summary(t,c);ga_end();}
// `bck_status s`: the text of BRepCheck::Print.
void tp_bck_status_case(std::istream& in){
 GaReader r{in};int s=r.integer();ga_captured([&]{BRepCheck::Print(BRepCheck_Status(s),std::cout);});ga_end();}
// `bck_add n (s)...`: the list after each BRepCheck::Add.
void tp_bck_add_case(std::istream& in){
 GaReader r{in};int n=r.integer();NCollection_List<BRepCheck_Status> l;
 for(int k=0;k<n;++k){BRepCheck::Add(l,BRepCheck_Status(r.integer()));tp_bck_list(l);}ga_end();}
// `bck_prec program kind a b`: PrecCurve of curve a (kind 0) or of pcurve a on surface
// b (kind 2), PrecSurface of surface a (kind 1).
void tp_bck_prec_case(std::istream& in){
 GaReader r{in};TpShapes t=tp_shapes(r);int kind=r.integer(),a=r.integer(),b=r.integer();
 if(kind==0)GA_TRY(ga_real(BRepCheck::PrecCurve(GeomAdaptor_Curve(t.curves.at(a)))))
 else if(kind==1)GA_TRY(ga_real(BRepCheck::PrecSurface(new GeomAdaptor_Surface(t.surfaces.at(a)))))
 else GA_TRY(ga_real(BRepCheck::PrecCurve(Adaptor3d_CurveOnSurface(new Geom2dAdaptor_Curve(t.pcurves.at(a)),new GeomAdaptor_Surface(t.surfaces.at(b))))));
 ga_end();}
// `bck_selfint program wire face e1 e2`: BRepCheck::SelfIntersection with the output
// edges starting as the table edges e1 and e2 (-1: null).
void tp_bck_selfint_case(std::istream& in){
 GaReader r{in};TpShapes t=tp_shapes(r);int w=r.integer(),f=r.integer(),i1=r.integer(),i2=r.integer();
 TopoDS_Edge e1=i1<0?TopoDS_Edge():TopoDS::Edge(t.shapes.at(i1)),e2=i2<0?TopoDS_Edge():TopoDS::Edge(t.shapes.at(i2));
 GA_TRY(ga_flag(BRepCheck::SelfIntersection(TopoDS::Wire(t.shapes.at(w)),TopoDS::Face(t.shapes.at(f)),e1,e2)));
 ga_int(tp_shape_index(t,e1));ga_int(tp_shape_index(t,e2));ga_end();}
// The reference curve of ValidateEdge: 0 curve c on [f, l]; 1 pcurve c on [f, l] on
// surface s.
occ::handle<Adaptor3d_Curve> tp_ble_reference(GaReader& r,const TpShapes& t){
 int kind=r.integer(),c=r.integer();double f=r.real(),l=r.real();
 if(kind==0)return new GeomAdaptor_Curve(t.curves.at(c),f,l);
 return new Adaptor3d_CurveOnSurface(new Geom2dAdaptor_Curve(t.pcurves.at(c),f,l),new GeomAdaptor_Surface(t.surfaces.at(r.integer())));}
// `ble_validate program reference (pc f l s) sameparameter script`: the codes 1
// SetExactMethod b, 2 SetParallel b, 3 SetControlPointsNumber n, 4
// SetExitIfToleranceExceeded t, 5 Process, 6 IsDone, 7 GetMaxDistance, 8
// CheckTolerance t, 9 UpdateTolerance t, 10 IsExactMethod, 11 IsParallel.
void tp_ble_validate_case(std::istream& in){
 GaReader r{in};TpShapes t=tp_shapes(r);occ::handle<Adaptor3d_Curve> ref=tp_ble_reference(r,t);
 int pc=r.integer();double f=r.real(),l=r.real();int s=r.integer();
 occ::handle<Adaptor3d_CurveOnSurface> other=new Adaptor3d_CurveOnSurface(new Geom2dAdaptor_Curve(t.pcurves.at(pc),f,l),new GeomAdaptor_Surface(t.surfaces.at(s)));
 BRepLib_ValidateEdge v(ref,other,r.flag());int n=r.integer();
 for(int k=0;k<n;++k){int code=r.integer();
  switch(code){
  case 1:v.SetExactMethod(r.flag());break;
  case 2:v.SetParallel(r.flag());break;
  case 3:v.SetControlPointsNumber(r.integer());break;
  case 4:{double x=r.real();GA_TRY(v.SetExitIfToleranceExceeded(x));break;}
  case 5:GA_TRY(v.Process());break;
  case 6:ga_flag(v.IsDone());break;
  case 7:ga_real(v.GetMaxDistance());break;
  case 8:{double x=r.real();GA_TRY(ga_flag(v.CheckTolerance(x)));break;}
  case 9:{double x=r.real();v.UpdateTolerance(x);ga_real(x);break;}
  case 10:ga_flag(v.IsExactMethod());break;
  default:ga_flag(v.IsParallel());break;}}
 ga_end();}
// The checker of one table shape in an analyzer: -1 for a null checker, else its
// statuses and, in table order, those of every shape it has statuses on.
void tp_bck_analyzer_result(const TpShapes& t,const BRepCheck_Analyzer& a,const TopoDS_Shape& s){
 const occ::handle<BRepCheck_Result>& c=a.Result(s);
 if(c.IsNull()){ga_int(-1);return;}
 tp_bck_list(c->Status());
 for(size_t j=0;j<t.shapes.size();++j)if(c->IsStatusOnShape(t.shapes[j])){ga_int(int(j));tp_bck_list(c->StatusOnShape(t.shapes[j]));}
 ga_int(-9);}
// `bck_analyze program index geomcontrols parallel exact`: the analyzer of shape
// `index`, its flags, IsValid, then for each table shape IsValid(S) and its checker.
void tp_bck_analyze_case(std::istream& in){
 GaReader r{in};TpShapes t=tp_shapes(r);int index=r.integer();bool g=r.flag(),par=r.flag(),ex=r.flag();
 try{
  BRepCheck_Analyzer a(t.shapes.at(index),g,par,ex);
  ga_flag(a.IsParallel());ga_flag(a.IsExactMethod());GA_TRY(ga_flag(a.IsValid()));
  for(const TopoDS_Shape& s:t.shapes){GA_TRY(ga_flag(a.IsValid(s)));GA_TRY(tp_bck_analyzer_result(t,a,s));}
  a.SetParallel(!par);a.SetExactMethod(!ex);ga_flag(a.IsParallel());ga_flag(a.IsExactMethod());
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
