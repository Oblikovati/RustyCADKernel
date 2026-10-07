// BRepFill_Pipe probe operations; the Rust twin is
// crates/tkbool/examples/bl_probe/brep_fill_pipe.rs.
//
// The spine and the profile are shapes of a shape program (tktopalgo_probe_shapes.cxx);
// shapes print through pr_shape (tkprim_probe_shape.cxx) and generated shapes through
// bl_list (tkbool_probe_brepfill_generator.cxx).
#include <BRepFill_Pipe.hxx>
// `program spine profile mode c1 x y z tables`: a BRepFill_Pipe of the profile along the
// spine with the trihedron mode (a GeomFill_Trihedron value) and ForceApproxC1; prints the
// shape, ErrorOnSurface and the first and last shapes, Generated of every shape of the
// program, with `tables` Face of every spine edge and profile edge, Edge of every spine
// edge and profile vertex and Section of every spine vertex (TopExp_Explorer orders;
// the tables are null natively for a vertex profile, a crash), then PipeLine of the
// point.
void bl_bf_pipe_case(std::istream& input){
 GaReader r{input};TpShapes t;
 try{t=tp_shapes(r);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 TopoDS_Wire spine=TopoDS::Wire(t.shapes.at(r.integer()));TopoDS_Shape profile=t.shapes.at(r.integer());
 GeomFill_Trihedron mode=(GeomFill_Trihedron)r.integer();bool c1=r.flag();gp_Pnt point=r.pnt();bool tables=r.flag();
 std::unique_ptr<BRepFill_Pipe> pipe;
 try{pipe.reset(new BRepFill_Pipe(spine,profile,mode,c1));}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';ga_end();return;}
 GA_TRY(pr_shape(pipe->Shape()))
 GA_TRY(ga_real(pipe->ErrorOnSurface()))
 GA_TRY(pr_shape(pipe->FirstShape()))
 GA_TRY(pr_shape(pipe->LastShape()))
 NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> map;
 if(!pipe->Shape().IsNull())TopExp::MapShapes(pipe->Shape(),map);
 for(const TopoDS_Shape& s:t.shapes){NCollection_List<TopoDS_Shape> l;GA_TRY(pipe->Generated(s,l);bl_list(map,l))}
 if(tables)for(TopExp_Explorer es(spine,TopAbs_EDGE);es.More();es.Next()){
  for(TopExp_Explorer ep(profile,TopAbs_EDGE);ep.More();ep.Next())
   GA_TRY(bl_indexed(map,pipe->Face(TopoDS::Edge(es.Current()),TopoDS::Edge(ep.Current()))))
  for(TopExp_Explorer ev(profile,TopAbs_VERTEX);ev.More();ev.Next())
   GA_TRY(bl_indexed(map,pipe->Edge(TopoDS::Edge(es.Current()),TopoDS::Vertex(ev.Current()))))}
 if(tables)for(TopExp_Explorer ev(spine,TopAbs_VERTEX);ev.More();ev.Next())
  GA_TRY(pr_shape(pipe->Section(TopoDS::Vertex(ev.Current()))))
 GA_TRY(pr_shape(pipe->PipeLine(point)))
 ga_end();}
