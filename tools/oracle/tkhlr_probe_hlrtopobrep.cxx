// HLRTopoBRep (OutLiner, DSFiller, FaceIsoLiner, Data), the Rust twin of
// crates/tkhlr/examples/hl_probe/hlrtopobrep.rs.
#include <HLRTopoBRep_OutLiner.hxx>
// Every edge of a shape in exploration order: its orientation and summary.
void hb_shape_edges(const TopoDS_Shape& s){
 int n=0;for(TopExp_Explorer ex(s,TopAbs_EDGE);ex.More();ex.Next())++n;ga_int(n);
 for(TopExp_Explorer ex(s,TopAbs_EDGE);ex.More();ex.Next()){ga_int(ex.Current().Orientation());
  GA_TRY(hb_edge_summary(TopoDS::Edge(ex.Current())));}}
// The faces and wires of a shape: per face its wire count and per wire its edge count.
void hb_shape_topology(const TopoDS_Shape& s){
 for(TopExp_Explorer ef(s,TopAbs_FACE);ef.More();ef.Next()){int nw=0;
  for(TopExp_Explorer ew(ef.Current(),TopAbs_WIRE);ew.More();ew.Next()){++nw;int ne=0;
   for(TopExp_Explorer ee(ew.Current(),TopAbs_EDGE);ee.More();ee.Next())++ne;ga_int(ne);}
  ga_int(-nw);}}
// The data structure queries on the faces and edges of the original shape.
void hb_topo_data(HLRTopoBRep_Data& ds,const TopoDS_Shape& s){
 for(TopExp_Explorer ef(s,TopAbs_FACE);ef.More();ef.Next()){const TopoDS_Face& f=TopoDS::Face(ef.Current());
  ga_flag(ds.FaceHasIntL(f));ga_flag(ds.FaceHasOutL(f));ga_flag(ds.FaceHasIsoL(f));
  GA_TRY(ga_int(ds.FaceIntL(f).Extent());ga_int(ds.FaceOutL(f).Extent());ga_int(ds.FaceIsoL(f).Extent()));}
 for(TopExp_Explorer ee(s,TopAbs_EDGE);ee.More();ee.Next()){const TopoDS_Edge& e=TopoDS::Edge(ee.Current());
  ga_flag(ds.EdgeHasSplE(e));GA_TRY(ga_int(ds.EdgeSplE(e).Extent()));
  TopoDS_Vertex v1,v2;TopExp::Vertices(e,v1,v2);
  for(const TopoDS_Vertex& v:{v1,v2})if(!v.IsNull()){ga_flag(ds.IsOutV(v));ga_flag(ds.IsIntV(v));}}}
// `projector shape nbIso`: OutLiner(S).Fill(P, MST, nbIso), then the outlined shape and
// the data structure.
void ht_outliner_case(std::istream& input){
 GaReader r{input};HLRAlgo_Projector projector=hb_projector(r);
 GA_TRY(TopoDS_Shape s=hb_shape(r);int nb_iso=r.integer();
  occ::handle<HLRTopoBRep_OutLiner> o=new HLRTopoBRep_OutLiner(s);
  NCollection_DataMap<TopoDS_Shape,BRepTopAdaptor_Tool,TopTools_ShapeMapHasher> mst;
  o->Fill(projector,mst,nb_iso);const TopoDS_Shape& out=o->OutLinedShape();
  ga_flag(out.IsNull());hb_shape_topology(out);hb_shape_edges(out);hb_topo_data(o->DataStructure(),s));
 ga_end();}
