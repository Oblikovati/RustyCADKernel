// BRepFont regularization and glyph/text builders, the twin of
// crates/tkv3d/examples/v3_probe/brep_font.rs.
#include <BRepCheck_Analyzer.hxx>
#include <BRepFont_Builder.hxx>
#include <BRepFont_Regularizer.hxx>
#include <BRepGraph.hxx>
#include <BRepGraph_EditorView.hxx>
#include <BRepGraph_RefsView.hxx>
#include <BRepGraph_TopoView.hxx>
#include <BRep_Tool.hxx>
#include <TopExp_Explorer.hxx>
#include <TopoDS.hxx>
#include <TopoDS_Iterator.hxx>
#include <gp_Ax3.hxx>
#include <gp_Trsf.hxx>
// Reads `Kind P0 P1 P2 P3` (8 coordinates): a line, quadratic or cubic outline segment.
Font_GlyphOutline::Segment v3_font_segment(GaReader& r){
 int kind=r.integer();gp_XY p[4];for(gp_XY& v:p){double x=r.real();v=gp_XY(x,r.real());}
 if(kind==0)return Font_GlyphOutline::Segment::Line(p[0],p[3]);
 if(kind==1)return Font_GlyphOutline::Segment::Quadratic(p[0],p[1],p[3]);
 return Font_GlyphOutline::Segment::Cubic(p[0],p[1],p[2],p[3]);}
// Reads `UnitsPerEm Rule Overlaps SingleStroke NbContours {NbSegments Segment...} NbOrphans
// Segment...`: packed contours followed by segments that no contour covers.
Font_GlyphOutline v3_font_outline(GaReader& r){
 double units=r.real();int rule=r.integer();bool overlaps=r.flag(),stroke=r.flag();
 NCollection_LinearVector<Font_GlyphOutline::Segment> segments;NCollection_LinearVector<Font_GlyphOutline::Contour> contours;
 int nbContours=r.integer();
 for(int c=0;c<nbContours;++c){int n=r.integer();uint32_t first=uint32_t(segments.Size());
  for(int s=0;s<n;++s)segments.Append(v3_font_segment(r));contours.EmplaceAppend(first,uint32_t(n));}
 int orphans=r.integer();for(int s=0;s<orphans;++s)segments.Append(v3_font_segment(r));
 return Font_GlyphOutline::Create(units,rule==0?Font_GlyphOutline::FillRule::NonZero:Font_GlyphOutline::FillRule::EvenOdd,
  overlaps,stroke,std::move(segments),std::move(contours));}
// A 2D box: the void flag, then the four limits with the gap.
void v3_font_box(const Bnd_Box2d& b){ga_flag(b.IsVoid());if(b.IsVoid())return;double x0,y0,x1,y1;b.Get(x0,y0,x1,y1);
 ga_real(x0);ga_real(y0);ga_real(x1);ga_real(y1);}
void v3_font_curve_state(const BRepFont_PlanarRegion::Curve& c){ga_int(int(c.Type()));ga_xy(c.Start());ga_xy(c.Control1());ga_xy(c.Control2());ga_xy(c.End());}
// Every pool of a region in order.
void v3_font_region(const BRepFont_PlanarRegion& g){
 ga_real(g.UnitsPerEm());ga_real(g.Tolerance());ga_flag(g.IsSingleStroke());ga_flag(g.IsEmpty());
 ga_int(int(g.Vertices().Size()));for(const gp_XY& v:g.Vertices())ga_xy(v);
 ga_int(int(g.Curves().Size()));for(const BRepFont_PlanarRegion::Curve& c:g.Curves())v3_font_curve_state(c);
 ga_int(int(g.Uses().Size()));for(const BRepFont_PlanarRegion::Use& u:g.Uses()){ga_int(int(u.CurveIndex().Value()));
  ga_int(int(u.FirstVertex().Value()));ga_int(int(u.LastVertex().Value()));ga_real(u.FirstParameter());ga_real(u.LastParameter());ga_flag(u.IsReversed());}
 ga_int(int(g.Loops().Size()));for(const BRepFont_PlanarRegion::Loop& l:g.Loops()){ga_int(int(l.FirstUse()));ga_int(int(l.NbUses()));
  ga_real(l.SignedArea());v3_font_box(l.Bounds());}
 ga_int(int(g.HoleLoops().Size()));for(const BRepFont_PlanarRegion::LoopId& h:g.HoleLoops())ga_int(int(h.Value()));
 ga_int(int(g.Regions().Size()));for(const BRepFont_PlanarRegion::Region& q:g.Regions()){ga_int(int(q.OuterLoop().Value()));
  ga_int(int(q.FirstHole()));ga_int(int(q.NbHoles()));}}
// `Outline Tolerance`: the status, then the region when one is returned.
void v3_brepfont_regularize(std::istream& input){
 GaReader r{input};Font_GlyphOutline outline=v3_font_outline(r);BRepFont_Regularizer::Options options;options.Tolerance=r.real();
 BRepFont_Regularizer::Result result=BRepFont_Regularizer().Build(outline,options);
 ga_int(int(result.StatusValue()));ga_flag(result.IsDone());ga_flag(result.PlanarRegion().has_value());
 if(result.PlanarRegion().has_value())v3_font_region(*result.PlanarRegion());
 ga_end();}
// `Segment NbParameters T...`: Value and D1 at each parameter, SignedArea and Bounds.
void v3_brepfont_curve(std::istream& input){
 GaReader r{input};BRepFont_PlanarRegion::Curve curve(v3_font_segment(r));int n=r.integer();
 for(int i=0;i<n;++i){double t=r.real();ga_xy(curve.Value(t));ga_xy(curve.D1(t));}
 ga_real(curve.SignedArea());v3_font_box(curve.Bounds());ga_end();}
int v3_font_count(const TopoDS_Shape& s,TopAbs_ShapeEnum t){int n=0;for(TopExp_Explorer e(s,t);e.More();e.Next())++n;return n;}
// The topology of a glyph shape: null flag, type, counts, validity, vertex points and edge ranges.
void v3_font_shape(const TopoDS_Shape& s){
 ga_flag(s.IsNull());if(s.IsNull())return;ga_int(int(s.ShapeType()));
 for(TopAbs_ShapeEnum t:{TopAbs_FACE,TopAbs_WIRE,TopAbs_EDGE,TopAbs_VERTEX})ga_int(v3_font_count(s,t));
 ga_flag(BRepCheck_Analyzer(s).IsValid());
 for(TopExp_Explorer e(s,TopAbs_VERTEX);e.More();e.Next())ga_pnt(BRep_Tool::Pnt(TopoDS::Vertex(e.Current())));
 for(TopExp_Explorer e(s,TopAbs_EDGE);e.More();e.Next()){double f,l;BRep_Tool::Range(TopoDS::Edge(e.Current()),f,l);
  ga_real(f);ga_real(l);ga_int(int(e.Current().Orientation()));}}
void v3_font_graph_counts(const BRepGraph& g){
 ga_int(int(g.Topo().Vertices().NbActive()));ga_int(int(g.Topo().Edges().NbActive()));ga_int(int(g.Topo().CoEdges().NbActive()));
 ga_int(int(g.Topo().Wires().NbActive()));ga_int(int(g.Topo().Faces().NbActive()));ga_int(int(g.Topo().Compounds().NbActive()));}
// `Outline RegularizerTolerance Size Tolerance Concatenate`: the regularized glyph built as a
// TopoDS shape, then added to a new graph.
void v3_brepfont_glyph(std::istream& input){
 GaReader r{input};Font_GlyphOutline outline=v3_font_outline(r);BRepFont_Regularizer::Options options;options.Tolerance=r.real();
 BRepFont_Builder::GlyphOptions glyph;glyph.Size=r.real();glyph.Tolerance=r.real();glyph.ToConcatenateContours=r.flag();
 BRepFont_Regularizer::Result result=BRepFont_Regularizer().Build(outline,options);
 ga_flag(result.PlanarRegion().has_value());
 if(result.PlanarRegion().has_value()){const BRepFont_PlanarRegion& region=*result.PlanarRegion();
  v3_font_shape(BRepFont_Builder::BuildGlyph(region,glyph));
  BRepGraph graph;BRepGraph_NodeId root=BRepFont_Builder::AddGlyph(graph,region,glyph);ga_flag(root.IsValid());
  if(root.IsValid())ga_int(int(root.NodeKind));v3_font_graph_counts(graph);}
 ga_end();}
void v3_font_trsf(const gp_Trsf& t){for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)ga_real(t.Value(i,j));}
// `Outline RegularizerTolerance Size Tolerance NbItems {Region X Y} Pen(9) Concatenate`: a plan
// whose items reuse one regularized region (Region 1 refers to an empty region), its TopoDS
// text and its graph text.
void v3_brepfont_text(std::istream& input){
 GaReader r{input};Font_GlyphOutline outline=v3_font_outline(r);BRepFont_Regularizer::Options options;options.Tolerance=r.real();
 double size=r.real(),tolerance=r.real();int nbItems=r.integer();
 NCollection_LinearVector<BRepFont_Builder::TextPlan::Item> items;
 for(int i=0;i<nbItems;++i){int region=r.integer();double x=r.real();items.EmplaceAppend(size_t(region),gp_XY(x,r.real()));}
 gp_Pnt location=r.pnt();gp_Dir normal=r.dir(),xDirection=r.dir();bool concatenate=r.flag();
 BRepFont_Regularizer::Result result=BRepFont_Regularizer().Build(outline,options);
 if(!result.PlanarRegion().has_value()){ga_flag(false);ga_end();return;}
 NCollection_LinearVector<std::shared_ptr<const BRepFont_PlanarRegion>> regions;
 regions.Append(std::make_shared<BRepFont_PlanarRegion>(std::move(*result.ChangePlanarRegion())));
 regions.Append(std::make_shared<BRepFont_PlanarRegion>(*BRepFont_Regularizer().Build(Font_GlyphOutline::Create(1000.0,
  Font_GlyphOutline::FillRule::NonZero,false,false,{},{})).PlanarRegion()));
 std::optional<BRepFont_Builder::TextPlan> plan=BRepFont_Builder::CreateTextPlan(size,tolerance,gp_XY(0,0),gp_XY(10,10),std::move(regions),std::move(items));
 ga_flag(plan.has_value());if(!plan.has_value()){ga_end();return;}
 BRepFont_Builder::TextOptions text;GA_TRY(text.Pen=gp_Ax3(location,normal,xDirection));text.ToConcatenateContours=concatenate;
 TopoDS_Shape shape=BRepFont_Builder::BuildText(*plan,text);ga_flag(shape.IsNull());
 if(!shape.IsNull()){ga_int(shape.NbChildren());for(TopoDS_Iterator it(shape);it.More();it.Next())v3_font_trsf(it.Value().Location().Transformation());}
 BRepGraph graph;BRepGraph_NodeId root=BRepFont_Builder::AddText(graph,*plan,text);ga_flag(root.IsValid());v3_font_graph_counts(graph);
 if(root.IsValid()){const NCollection_LinearVector<BRepGraph_ChildRefId>& refs=graph.Refs().Children().IdsOf(BRepGraph_CompoundId(root));
  ga_int(int(refs.Size()));for(const BRepGraph_ChildRefId& ref:refs){TopLoc_Location l=graph.Refs().Gen().LocalLocation(ref);
   ga_flag(l.IsIdentity());v3_font_trsf(l.Transformation());}}
 ga_end();}
