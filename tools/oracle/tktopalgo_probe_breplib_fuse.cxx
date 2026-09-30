// BRepLib_FuseEdges probe operation and the shape records 10 and 11; the Rust twin is
// crates/tktopalgo/examples/tp_probe/brep_lib_fuse.rs.
//
// Shape records (continuing tktopalgo_probe_breplib_face.cxx):
// 10 the BRepLib_MakeFace face of a wire record and `onlyplane` (a null shape when not
//   done);
// 11 a wire of pieces of one Geom curve `curve nv (u)... ne (i j a b)...`: vertices at the
//   curve points of the parameters (tolerance 1e-7), then BRepLib_MakeEdge(C, Vi, Vj, a,
//   b) edges (left out when not done) added to a BRep_Builder wire.
//
// `blp_fuseedges shape concat n (edge index)... mode`: BRepLib_FuseEdges on the shape,
//   SetConcatBSpl when `concat`, AvoidEdges with the edges of the indices (MapShapes
//   order). Mode 0: the chains (count, then per key 1..count the edge indices and
//   orientations), NbVertices, the result edge digests (-1 unbound), the shape digest
//   and, per face of the input (MapShapes order), whether Faces() maps it; mode 1:
//   Perform() first, then the shape digest; mode 2: NbVertices then the shape digest.
#include <BRepLib_FuseEdges.hxx>
TopoDS_Shape tp_shape_record_fuse(GaReader& r,int kind){
 BRep_Builder B;
 if(kind==10){TopoDS_Shape w=tp_shape_record(r);BRepLib_MakeFace m(TopoDS::Wire(w),r.flag());
  if(!m.IsDone())return TopoDS_Shape();return m.Face();}
 occ::handle<Geom_Curve> c=tp_geom_curve(r);int nv=r.integer();std::vector<TopoDS_Vertex> vs;
 for(int i=0;i<nv;++i){TopoDS_Vertex v;B.MakeVertex(v,c->Value(r.real()),1e-7);vs.push_back(v);}
 TopoDS_Wire w;B.MakeWire(w);int ne=r.integer();
 for(int i=0;i<ne;++i){int a=r.integer(),b=r.integer();double p1=r.real(),p2=r.real();
  BRepLib_MakeEdge m(c,vs.at(a),vs.at(b),p1,p2);if(m.IsDone())B.Add(w,m.Edge());}
 return w;}
void tp_fuse_chains(BRepLib_FuseEdges& fe,const TpShapeMap& edges,int& count){
 NCollection_DataMap<int,NCollection_List<TopoDS_Shape>> m;fe.Edges(m);count=m.Extent();ga_int(count);
 for(int k=1;k<=count;++k){if(!m.IsBound(k)){ga_int(-1);continue;}const NCollection_List<TopoDS_Shape>& l=m(k);ga_int(l.Extent());
  for(NCollection_List<TopoDS_Shape>::Iterator it(l);it.More();it.Next()){ga_int(edges.FindIndex(it.Value()));ga_int(it.Value().Orientation());}}}
void tp_fuse_results(BRepLib_FuseEdges& fe,const TopoDS_Shape& s,int count){
 NCollection_DataMap<int,TopoDS_Shape> re;fe.ResultEdges(re);
 for(int k=1;k<=count;++k){if(re.IsBound(k))tp_shape(re(k));else ga_int(-1);}
 tp_shape(fe.Shape());
 NCollection_DataMap<TopoDS_Shape,TopoDS_Shape,TopTools_ShapeMapHasher> fm;fe.Faces(fm);TpShapeMap faces=tp_sub_shapes(s,TopAbs_FACE);
 for(int i=1;i<=faces.Extent();++i)ga_flag(fm.IsBound(faces(i)));}
void tp_blp_fuseedges_case(std::istream& input){
 GaReader r{input};
 try{TopoDS_Shape s=tp_shape_record(r);bool concat=r.flag();int n=r.integer();TpShapeMap edges=tp_sub_shapes(s,TopAbs_EDGE);
  TpShapeMap avoid;for(int i=0;i<n;++i)avoid.Add(edges(r.integer()));int mode=r.integer();
  BRepLib_FuseEdges fe(s);if(concat)fe.SetConcatBSpl();fe.AvoidEdges(avoid);
  if(mode==1){fe.Perform();tp_shape(fe.Shape());}
  else if(mode==2){ga_int(fe.NbVertices());tp_shape(fe.Shape());}
  else{int count=0;tp_fuse_chains(fe,edges,count);ga_int(fe.NbVertices());tp_fuse_results(fe,s,count);}
 }catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
