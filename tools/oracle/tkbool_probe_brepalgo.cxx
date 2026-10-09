// BRepAlgo package probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_algo.rs.
//
// The shapes are shapes of a shape program (tktopalgo_probe_shapes.cxx); shapes print
// through pr_shape (tkprim_probe_shape.cxx).
#include <BRepAlgo.hxx>
#include <BRepAlgo_FaceRestrictor.hxx>
#include <BRepAlgo_Loop.hxx>
// `program wire option angtol`: ConcatenateWire (option as a GeomAbs_Shape value).
void bl_ba_concat_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Wire w=TopoDS::Wire(t.shapes.at(r.integer()));GeomAbs_Shape option=(GeomAbs_Shape)r.integer();double tol=r.real();
 GA_TRY(pr_shape(BRepAlgo::ConcatenateWire(w,option,tol)))
 ga_end();}
// `program wire`: ConcatenateWireC0.
void bl_ba_concat_c0_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Wire w=TopoDS::Wire(t.shapes.at(r.integer()));
 GA_TRY(pr_shape(BRepAlgo::ConcatenateWireC0(w)))
 ga_end();}
// `program face angtol`: ConvertWire of each wire of the face, then ConvertFace.
void bl_ba_convert_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Face f=TopoDS::Face(t.shapes.at(r.integer()));double tol=r.real();
 for(TopExp_Explorer ex(f,TopAbs_WIRE);ex.More();ex.Next())GA_TRY(pr_shape(BRepAlgo::ConvertWire(TopoDS::Wire(ex.Current()),tol,f)))
 GA_TRY(pr_shape(BRepAlgo::ConvertFace(f,tol)))
 ga_end();}
// `program n arg... result closed geomctrl`: IsValid and IsTopologicallyValid of the
// result, then IsValid of the result of the arguments.
void bl_ba_valid_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 NCollection_List<TopoDS_Shape> args;int n=r.integer();for(int i=0;i<n;i++)args.Append(t.shapes.at(r.integer()));
 const TopoDS_Shape res=t.shapes.at(r.integer());bool closed=r.flag(),geom=r.flag();
 GA_TRY(ga_flag(BRepAlgo::IsValid(res)))
 GA_TRY(ga_flag(BRepAlgo::IsTopologicallyValid(res)))
 GA_TRY(ga_flag(BRepAlgo::IsValid(args,res,closed,geom)))
 ga_end();}
// `program face proj control n wire...`: a FaceRestrictor on the face with the wires:
// Perform, IsDone, each face, then the face count.
void bl_ba_face_restrictor_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Face f=TopoDS::Face(t.shapes.at(r.integer()));bool proj=r.flag(),control=r.flag();int n=r.integer();
 BRepAlgo_FaceRestrictor fr;fr.Init(f,proj,control);
 for(int i=0;i<n;i++){TopoDS_Wire w=TopoDS::Wire(t.shapes.at(r.integer()));fr.Add(w);}
 GA_TRY(fr.Perform())
 ga_flag(fr.IsDone());
 int count=0;
 GA_TRY(for(;fr.More();fr.Next()){pr_shape(fr.Current());count++;})
 ga_int(count);
 ga_end();}
// A list of shapes of the table: `n index...` with an orientation per index (-1 as is).
NCollection_List<TopoDS_Shape> bl_shape_list(GaReader& r,const TpShapes& t){
 NCollection_List<TopoDS_Shape> l;int n=r.integer();
 for(int i=0;i<n;i++){TopoDS_Shape s=t.shapes.at(r.integer());int o=r.integer();if(o>=0)s.Orientation((TopAbs_Orientation)o);l.Append(s);}
 return l;}
// The pieces of a cut edge: count, then per piece its range, orientation and the table
// indices (IsSame) of its vertices.
void bl_ba_pieces(const NCollection_List<TopoDS_Shape>& ne,const TpShapes& t){
 ga_int(ne.Extent());
 for(NCollection_List<TopoDS_Shape>::Iterator it(ne);it.More();it.Next()){
  const TopoDS_Edge e=TopoDS::Edge(it.Value());double f,l;BRep_Tool::Range(e,f,l);ga_real(f);ga_real(l);ga_int(e.Orientation());
  TopoDS_Vertex v1,v2;TopExp::Vertices(e,v1,v2);ga_int(tp_same_index(t,v1));ga_int(tp_same_index(t,v2));}}
// `program face edge list`: CutEdge of the edge at the listed vertices.
void bl_ba_cut_edge_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Face f=TopoDS::Face(t.shapes.at(r.integer()));const TopoDS_Edge e=TopoDS::Edge(t.shapes.at(r.integer()));
 NCollection_List<TopoDS_Shape> lv=bl_shape_list(r,t);
 BRepAlgo_Loop lp;lp.Init(f);
 GA_TRY(NCollection_List<TopoDS_Shape> ne;lp.CutEdge(e,lv,ne);bl_ba_pieces(ne,t))
 ga_end();}
// `program face n (edge list)... consts tolconf`: a Loop with the cut edges and the
// constant edges: Perform, the new wires, WiresToFaces and the new faces, the new edges
// of each cut edge, and the substitute count.
void bl_ba_loop_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 const TopoDS_Face f=TopoDS::Face(t.shapes.at(r.integer()));int n=r.integer();
 BRepAlgo_Loop lp;lp.Init(f);std::vector<TopoDS_Edge> cut;
 for(int i=0;i<n;i++){TopoDS_Edge e=TopoDS::Edge(t.shapes.at(r.integer()));NCollection_List<TopoDS_Shape> lv=bl_shape_list(r,t);lp.AddEdge(e,lv);cut.push_back(e);}
 NCollection_List<TopoDS_Shape> consts=bl_shape_list(r,t);lp.AddConstEdges(consts);
 lp.SetTolConf(r.real());ga_real(lp.GetTolConf());
 GA_TRY(lp.Perform())
 ga_int(lp.NewWires().Extent());
 for(NCollection_List<TopoDS_Shape>::Iterator it(lp.NewWires());it.More();it.Next())pr_shape(it.Value());
 GA_TRY(lp.WiresToFaces())
 ga_int(lp.NewFaces().Extent());
 for(NCollection_List<TopoDS_Shape>::Iterator it(lp.NewFaces());it.More();it.Next())pr_shape(it.Value());
 for(const TopoDS_Edge& e:cut)GA_TRY(bl_ba_pieces(lp.NewEdges(e),t))
 GA_TRY(lp.NewEdges(TopoDS::Edge(t.shapes.at(r.integer()))))
 NCollection_DataMap<TopoDS_Shape,TopoDS_Shape,TopTools_ShapeMapHasher> vv;lp.GetVerticesForSubstitute(vv);ga_int(vv.Extent());
 ga_end();}
