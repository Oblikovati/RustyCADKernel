// BRepOffset_Analyse probe operations; the Rust twin is
// crates/tkoffset/examples/of_probe/analyse.rs.
//
// The shapes are primitives (ot_primitive of tkoffset_probe_brepoffset_tool.cxx); the
// artificial faces print through pr_shape (tkprim_probe_shape.cxx).
#include <BRepOffset_Analyse.hxx>
#include <BRepOffset_Interval.hxx>
#include <TopoDS_Compound.hxx>
// The map index and orientation of a shape (0 when it is not in the map).
static void oan_ref(const NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher>& all,const TopoDS_Shape& s){
 ga_int(all.FindIndex(s));ga_int(s.Orientation());}
// `oan_analyse prim angle offset n (face offset)...`: BRepOffset_Analyse of the primitive
// with the offset value and the offsets of the listed faces (explorer indices). The
// shapes are indexed in the map of the primitive extended by the sub-shapes of the new
// faces. Prints IsDone, the new faces (index, orientation, shape), then for every shape
// of the map the intervals of an edge (Type), its ancestors, its generated shape and
// its descendants, then EdgeReplacement of every edge of every face of the primitive, the
// groups of Explode for each concavity type and pair of types, HasAncestor and
// HasGenerated of every shape, Edges of every vertex and face of the primitive for each
// type, and TangentEdges of every edge at each of its vertices.
void of_oan_analyse_case(std::istream& input){
 GaReader r{input};
 try{
  TopoDS_Shape p=ot_primitive(r);double angle=r.real(),off=r.real();int n=r.integer();
  NCollection_DataMap<TopoDS_Shape,double,TopTools_ShapeMapHasher> offsets;
  for(int i=0;i<n;i++){TopoDS_Shape f=ot_sub(p,TopAbs_FACE,r.integer());double v=r.real();offsets.Bind(f,v);}
  BRepOffset_Analyse a;a.SetOffsetValue(off);a.SetFaceOffsetMap(offsets);a.Perform(p,angle);
  ga_flag(a.IsDone());
  NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> all;TopExp::MapShapes(p,all);
  ga_int(a.NewFaces().Extent());
  for(const TopoDS_Shape& nf:a.NewFaces()){TopExp::MapShapes(nf,all);oan_ref(all,nf);pr_shape(nf);}
  for(int i=1;i<=all.Extent();i++){
   const TopoDS_Shape& s=all(i);
   if(s.ShapeType()==TopAbs_EDGE)GA_TRY(
    const NCollection_List<BRepOffset_Interval>& l=a.Type(TopoDS::Edge(s));ga_int(l.Extent());
    for(const BRepOffset_Interval& iv:l){ga_real(iv.First());ga_real(iv.Last());ga_int(iv.Type());})
   GA_TRY(const NCollection_List<TopoDS_Shape>& l=a.Ancestors(s);ga_int(l.Extent());for(const TopoDS_Shape& x:l)oan_ref(all,x);)
   const TopoDS_Shape g=a.Generated(s);if(g.IsNull())ga_int(-1);else oan_ref(all,g);
   const NCollection_List<TopoDS_Shape>* d=a.Descendants(s);
   if(!d)ga_int(-1);else{ga_int(d->Extent());for(const TopoDS_Shape& x:*d)oan_ref(all,x);}}
  for(TopExp_Explorer ef(p,TopAbs_FACE);ef.More();ef.Next())
   for(TopExp_Explorer ee(ef.Current(),TopAbs_EDGE);ee.More();ee.Next())
    oan_ref(all,a.EdgeReplacement(TopoDS::Face(ef.Current()),TopoDS::Edge(ee.Current())));
  for(int t1=0;t1<=5;t1++)for(int t2=t1;t2<=5;t2++){
   NCollection_List<TopoDS_Shape> groups;
   GA_TRY(if(t1==t2)a.Explode(groups,ChFiDS_TypeOfConcavity(t1));else a.Explode(groups,ChFiDS_TypeOfConcavity(t1),ChFiDS_TypeOfConcavity(t2));
    ga_int(groups.Extent());
    for(const TopoDS_Shape& g:groups){NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> fm;TopExp::MapShapes(g,TopAbs_FACE,fm);
     ga_int(fm.Extent());for(int k=1;k<=fm.Extent();k++)oan_ref(all,fm(k));})}
  for(int i=1;i<=all.Extent();i++){ga_flag(a.HasAncestor(all(i)));ga_flag(a.HasGenerated(all(i)));}
  for(int t=0;t<=5;t++){
   for(TopExp_Explorer ev(p,TopAbs_VERTEX);ev.More();ev.Next()){
    NCollection_List<TopoDS_Shape> l;GA_TRY(a.Edges(TopoDS::Vertex(ev.Current()),ChFiDS_TypeOfConcavity(t),l);ga_int(l.Extent());for(const TopoDS_Shape& x:l)oan_ref(all,x);)}
   for(TopExp_Explorer ef(p,TopAbs_FACE);ef.More();ef.Next()){
    NCollection_List<TopoDS_Shape> l;GA_TRY(a.Edges(TopoDS::Face(ef.Current()),ChFiDS_TypeOfConcavity(t),l);ga_int(l.Extent());for(const TopoDS_Shape& x:l)oan_ref(all,x);)}}
  for(TopExp_Explorer ee(p,TopAbs_EDGE);ee.More();ee.Next())
   for(TopExp_Explorer ev(ee.Current(),TopAbs_VERTEX);ev.More();ev.Next()){
    NCollection_List<TopoDS_Shape> l;GA_TRY(a.TangentEdges(TopoDS::Edge(ee.Current()),TopoDS::Vertex(ev.Current()),l);ga_int(l.Extent());for(const TopoDS_Shape& x:l)oan_ref(all,x);)}}
 catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<' ';}
 ga_end();}
