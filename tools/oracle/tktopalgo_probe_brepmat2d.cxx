// BRepMAT2d probe operation over faces read by tp_shape (tktopalgo_probe_shape.cxx); the
// Rust twin is crates/tktopalgo/examples/tp_probe/brep_mat2d.rs.
//
// `bm2_locus shape nw (closed)... side join open line link`: the wires of the face (in
// TopExp_Explorer order) get the given Closed flags (the reader leaves them open), then
// BRepMAT2d_Explorer(face) and BRepMAT2d_BisectingLocus::Compute(explorer, line, side, join,
// open) run, and with `link` BRepMAT2d_LinkTopoBilo. Printed: the explorer (contours, their
// curves, the closed flags, the modified face, wires and edges), the locus (IsDone, counts,
// sections, the graph with the geometry of its arcs, nodes and basic elements) and the links
// of every edge and vertex of the face.
#include <BRepMAT2d_BisectingLocus.hxx>
#include <BRepMAT2d_Explorer.hxx>
#include <BRepMAT2d_LinkTopoBilo.hxx>
#include <Bisector_Bisec.hxx>
#include <Geom2d_CartesianPoint.hxx>
#include <Geom2d_TrimmedCurve.hxx>
#include <MAT_Arc.hxx>
#include <MAT_BasicElt.hxx>
#include <MAT_Graph.hxx>
#include <MAT_Node.hxx>
typedef NCollection_IndexedMap<TopoDS_Shape,TopTools_ShapeMapHasher> TpBm2Map;
// A trimmed 2d curve: its type, its basis type, its range and three points.
void tp_bm2_curve(const occ::handle<Geom2d_Curve>& c){
 tp_type(c);occ::handle<Geom2d_TrimmedCurve> t=occ::down_cast<Geom2d_TrimmedCurve>(c);if(!t.IsNull())tp_type(t->BasisCurve());
 double a=c->FirstParameter(),b=c->LastParameter();ga_real(a);ga_real(b);tp_span(a,b);
 for(int k=0;k<=4;k+=2)GA_TRY(ga_pnt2d(c->Value(tp_sample(a,b,k))))}
// A geometry of a basic element: a point, or a curve.
void tp_bm2_geometry(const occ::handle<Geom2d_Geometry>& g){
 occ::handle<Geom2d_CartesianPoint> p=occ::down_cast<Geom2d_CartesianPoint>(g);
 if(!p.IsNull()){ga_int(0);ga_pnt2d(p->Pnt2d());return;}
 ga_int(1);tp_bm2_curve(occ::down_cast<Geom2d_Curve>(g));}
// The explorer: its contours and curves, the closed flags, then IsModified of the face, of
// each wire and each edge (with the range and end points of the new pcurve on the new face).
void tp_bm2_explorer(BRepMAT2d_Explorer& e,const TopoDS_Shape& face){
 ga_int(e.NumberOfContours());
 for(int i=1;i<=e.NumberOfContours();i++){ga_int(e.NumberOfCurves(i));ga_flag(e.GetIsClosed()(i));
  for(e.Init(i);e.More();e.Next())tp_bm2_curve(e.Value());}
 ga_flag(e.IsModified(face));TopoDS_Face nf=TopoDS::Face(e.ModifiedShape(face));ga_int(nf.NbChildren());
 for(TopExp_Explorer w(face,TopAbs_WIRE);w.More();w.Next()){ga_flag(e.IsModified(w.Current()));
  ga_int(e.ModifiedShape(w.Current()).NbChildren());}
 for(TopExp_Explorer x(face,TopAbs_EDGE);x.More();x.Next()){const TopoDS_Shape& s=x.Current();ga_flag(e.IsModified(s));
  if(!e.IsModified(s))continue;TopoDS_Edge ne=TopoDS::Edge(e.ModifiedShape(s));ga_int(ne.Orientation());
  double f=0,l=0;occ::handle<Geom2d_Curve> c=BRep_Tool::CurveOnSurface(ne,nf,f,l);tp_type(c);ga_real(f);ga_real(l);
  if(!c.IsNull()){GA_TRY(ga_pnt2d(c->Value(f)))GA_TRY(ga_pnt2d(c->Value(l)))}
  double g=0,h=0;ga_flag(!BRep_Tool::Curve(ne,g,h).IsNull());}}
int tp_bm2_index(const occ::handle<MAT_Arc>& a){return a.IsNull()?-1:a->Index();}
int tp_bm2_index(const occ::handle<MAT_Node>& a){return a.IsNull()?-1:a->Index();}
int tp_bm2_index(const occ::handle<MAT_BasicElt>& a){return a.IsNull()?-1:a->Index();}
// The graph through the locus: arcs (GeomBis with Reverse), nodes (GeomElt) and basic
// elements (GeomElt), then BasicElt(line, i) of each line.
void tp_bm2_graph(const BRepMAT2d_BisectingLocus& b,const BRepMAT2d_Explorer& e){
 occ::handle<MAT_Graph> g=b.Graph();
 ga_int(g->NumberOfArcs());ga_int(g->NumberOfNodes());ga_int(g->NumberOfBasicElts());ga_int(g->NumberOfInfiniteNodes());
 for(int i=1;i<=g->NumberOfArcs();i++){GA_TRY(occ::handle<MAT_Arc> a=g->Arc(i);ga_int(a->Index());ga_int(a->GeomIndex());
  ga_int(tp_bm2_index(a->FirstElement()));ga_int(tp_bm2_index(a->SecondElement()));ga_int(tp_bm2_index(a->FirstNode()));
  ga_int(tp_bm2_index(a->SecondNode()));GA_TRY(bool rev=false;Bisector_Bisec bis=b.GeomBis(a,rev);ga_flag(rev);tp_bm2_curve(bis.Value())))}
 for(int i=1;i<=g->NumberOfNodes();i++){GA_TRY(occ::handle<MAT_Node> n=g->Node(i);ga_int(n->Index());ga_real(n->Distance());
  ga_flag(n->Infinite());GA_TRY(ga_pnt2d(b.GeomElt(n))))}
 for(int i=1;i<=g->NumberOfBasicElts();i++){GA_TRY(occ::handle<MAT_BasicElt> x=g->BasicElt(i);ga_int(x->Index());
  ga_int(x->GeomIndex());GA_TRY(tp_bm2_geometry(b.GeomElt(x))))}
 for(int l=1;l<=b.NumberOfContours();l++){GA_TRY(int n=b.NumberOfElts(l);ga_int(n);
  for(int i=1;i<=n;i++)GA_TRY(ga_int(tp_bm2_index(b.BasicElt(l,i)))))
  for(int k=1;k<=e.NumberOfCurves(l)+1;k++)GA_TRY(ga_int(b.NumberOfSections(l,k)))}}
// The links: for every edge and vertex of the face (MapShapes order) its basic elements and
// the generating shape of each (index in the map and orientation).
void tp_bm2_link(const BRepMAT2d_LinkTopoBilo& constLink,const TopoDS_Shape& face){
 BRepMAT2d_LinkTopoBilo link=constLink;TpBm2Map m;TopExp::MapShapes(face,m);
 for(int i=1;i<=m.Extent();i++){const TopoDS_Shape& s=m(i);
  if(s.ShapeType()!=TopAbs_EDGE&&s.ShapeType()!=TopAbs_VERTEX)continue;
  link.Init(s);int n=0;for(;link.More();link.Next()){occ::handle<MAT_BasicElt> x=link.Value();ga_int(x->Index());n++;
   GA_TRY(TopoDS_Shape gs=link.GeneratingShape(x);ga_int(m.FindIndex(gs));ga_int(gs.Orientation()))}
  ga_int(n);}}
void tp_bm2_locus_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpShape s=tp_shape(r);int nw=r.integer();TopExp_Explorer w(s.shape,TopAbs_WIRE);
  for(int i=0;i<nw;i++){bool c=r.flag();if(w.More()){TopoDS_Shape x=w.Current();x.Closed(c);w.Next();}}
  MAT_Side side=r.integer()==0?MAT_Left:MAT_Right;GeomAbs_JoinType join=tp_join(r.integer());bool open=r.flag();
  int line=r.integer();bool withLink=r.flag();
  BRepMAT2d_Explorer e(TopoDS::Face(s.shape));tp_bm2_explorer(e,s.shape);
  BRepMAT2d_BisectingLocus b;GA_TRY(b.Compute(e,line,side,join,open))ga_flag(b.IsDone());ga_int(b.NumberOfContours());
  if(b.IsDone()){tp_bm2_graph(b,e);
   if(withLink)GA_TRY(BRepMAT2d_LinkTopoBilo link(e,b);link.Init(TopoDS_Shape());tp_bm2_link(link,s.shape))})
 ga_end();}
