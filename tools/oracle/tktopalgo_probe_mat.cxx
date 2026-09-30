// MAT and MAT2d probe operations; the Rust twin is crates/tktopalgo/examples/tp_probe/mat.rs
// (the pipeline) and tp_probe/mat_parts.rs (the classes one by one).
//
// A figure is `nlines (closed nitems <curve>*)*`, each curve read by tp_curve2d
// (tktopalgo_probe_bisector.cxx) and trimmed (kind 7), as BRepMAT2d_Explorer gives them.
// The pipeline is the one of BRepMAT2d_BisectingLocus::Compute (TKTopAlgo, not ported yet),
// written again here: CutSketch, MAT2d_Circuit, MAT2d_Tool2d, MAT2d_Mat2d, MAT_Graph and the
// fusion of the basic elements of several lines.
#include <MAT2d_BiInt.hxx>
#include <MAT2d_Circuit.hxx>
#include <MAT2d_Connexion.hxx>
#include <MAT2d_CutCurve.hxx>
#include <MAT2d_Mat2d.hxx>
#include <MAT2d_MiniPath.hxx>
#include <MAT2d_Tool2d.hxx>
#include <MAT_Arc.hxx>
#include <MAT_BasicElt.hxx>
#include <MAT_Bisector.hxx>
#include <MAT_Edge.hxx>
#include <MAT_Graph.hxx>
#include <MAT_ListOfBisector.hxx>
#include <MAT_ListOfEdge.hxx>
#include <MAT_Node.hxx>
#include <MAT_Zone.hxx>
#include <NCollection_DataMap.hxx>
typedef NCollection_Sequence<occ::handle<Geom2d_Geometry>> TpLine;
typedef NCollection_Sequence<TpLine> TpFigure;
struct TpFigureInput{TpFigure figure;NCollection_Sequence<bool> closed;};
TpFigureInput tp_read_figure(GaReader& r){
 TpFigureInput f;int n=r.integer();
 for(int i=0;i<n;++i){f.closed.Append(r.flag());int m=r.integer();TpLine line;for(int j=0;j<m;++j)line.Append(tp_curve2d(r));f.figure.Append(line);}
 return f;}
// The static CutSketch of BRepMAT2d_BisectingLocus.cxx.
void tp_cut_sketch(TpFigure& figure,NCollection_DataMap<MAT2d_BiInt,int>& nbSect){
 MAT2d_CutCurve cuter;
 for(int i=1;i<=figure.Length();i++){TpLine& contour=figure.ChangeValue(i);int iCurveInit=0;
  for(int j=1;j<=contour.Length();j++){iCurveInit++;cuter.Perform(occ::down_cast<Geom2d_Curve>(contour.ChangeValue(j)));int nbSection=1;
   if(!cuter.UnModified()){int ico=j;nbSection=cuter.NbCurves();for(int k=1;k<=nbSection;k++){contour.InsertAfter(j,cuter.Value(k));j++;}contour.Remove(ico);j--;}
   nbSect.Bind(MAT2d_BiInt(i,iCurveInit),nbSection);}}}
int tp_index(const occ::handle<MAT_Arc>& a){return a.IsNull()?-1:a->Index();}
int tp_index(const occ::handle<MAT_Node>& a){return a.IsNull()?-1:a->Index();}
int tp_index(const occ::handle<MAT_BasicElt>& a){return a.IsNull()?-1:a->Index();}
void tp_connexion(const occ::handle<MAT2d_Connexion>& c){
 ga_int(c->IndexFirstLine());ga_int(c->IndexSecondLine());ga_int(c->IndexItemOnFirst());ga_int(c->IndexItemOnSecond());
 ga_real(c->ParameterOnFirst());ga_real(c->ParameterOnSecond());ga_pnt2d(c->PointOnFirst());ga_pnt2d(c->PointOnSecond());ga_real(c->Distance());}
// The circuit: its items, the lengths of the lines, the places of each item and the connexions.
void tp_circuit(const occ::handle<MAT2d_Circuit>& c,int nlines){
 ga_int(c->NumberOfItems());
 for(int i=1;i<=c->NumberOfItems();i++){occ::handle<Geom2d_Geometry> g=c->Value(i);ga_text(g->DynamicType()->Name());
  occ::handle<Geom2d_Curve> cu=occ::down_cast<Geom2d_Curve>(g);if(!cu.IsNull()){ga_real(cu->FirstParameter());ga_real(cu->LastParameter());}
  ga_flag(c->ConnexionOn(i));if(c->ConnexionOn(i))tp_connexion(c->Connexion(i));}
 for(int l=1;l<=nlines;l++){GA_TRY(int n=c->LineLength(l);ga_int(n);
  for(int k=1;k<=n;k++){GA_TRY(const NCollection_Sequence<int>& s=c->RefToEqui(l,k);ga_int(s.Length());for(int e:s)ga_int(e))})}
 GA_TRY(ga_int(c->LineLength(nlines+2)))}
// A geometric bisector of the tool: its range and three points.
void tp_geom_bis(const MAT2d_Tool2d& tool,int index){
 GA_TRY(occ::handle<Geom2d_TrimmedCurve> t=tool.GeomBis(index).Value();ga_text(t->BasisCurve()->DynamicType()->Name());
  double a=t->FirstParameter(),b=t->LastParameter();ga_real(a);ga_real(b);tp_span(a,b);
  for(int k=0;k<=4;k+=2){GA_TRY(ga_pnt2d(t->Value(tp_sample(a,b,k))))})}
// The graph: its counts, arcs, nodes, basic elements and zones.
void tp_graph(const occ::handle<MAT_Graph>& g,const MAT2d_Tool2d& tool){
 ga_int(g->NumberOfArcs());ga_int(g->NumberOfNodes());ga_int(g->NumberOfBasicElts());ga_int(g->NumberOfInfiniteNodes());
 for(int i=1;i<=g->NumberOfArcs();i++){GA_TRY(occ::handle<MAT_Arc> a=g->Arc(i);ga_int(a->Index());ga_int(a->GeomIndex());
  ga_int(tp_index(a->FirstElement()));ga_int(tp_index(a->SecondElement()));ga_int(tp_index(a->FirstNode()));ga_int(tp_index(a->SecondNode()));
  for(int s=0;s<2;s++){MAT_Side side=s==0?MAT_Left:MAT_Right;GA_TRY(ga_flag(a->HasNeighbour(a->FirstNode(),side));ga_int(tp_index(a->Neighbour(a->FirstNode(),side))))}
  tp_geom_bis(tool,a->GeomIndex()))}
 for(int i=1;i<=g->NumberOfNodes();i++){GA_TRY(occ::handle<MAT_Node> n=g->Node(i);ga_int(n->Index());ga_int(n->GeomIndex());ga_real(n->Distance());
  GA_TRY(ga_flag(n->PendingNode()));ga_flag(n->OnBasicElt());ga_flag(n->Infinite());GA_TRY(ga_pnt2d(tool.GeomPnt(n->GeomIndex())))
  GA_TRY(NCollection_Sequence<occ::handle<MAT_Arc>> s;n->LinkedArcs(s);ga_int(s.Length());for(const occ::handle<MAT_Arc>& a:s)ga_int(tp_index(a)))
  GA_TRY(NCollection_Sequence<occ::handle<MAT_BasicElt>> s;n->NearElts(s);ga_int(s.Length());for(const occ::handle<MAT_BasicElt>& e:s)ga_int(tp_index(e))))}
 for(int i=1;i<=g->NumberOfBasicElts();i++){GA_TRY(occ::handle<MAT_BasicElt> e=g->BasicElt(i);ga_int(e->Index());ga_int(e->GeomIndex());
  ga_int(tp_index(e->StartArc()));ga_int(tp_index(e->EndArc()));
  // A zone that reaches an infinite node turns from StartArc: a null one crashes natively.
  if(!e->StartArc().IsNull())GA_TRY(occ::handle<MAT_Zone> z=new MAT_Zone(e);ga_flag(z->NoEmptyZone());ga_flag(z->Limited());ga_int(z->NumberOfArcs());
   for(int k=1;k<=z->NumberOfArcs();k++)ga_int(tp_index(z->ArcOnFrontier(k)))))}}
// BRepMAT2d_BisectingLocus::RenumerationAndFusion.
void tp_fusion(const occ::handle<MAT_Graph>& graph,MAT2d_Tool2d& tool,int iline,int length,int& indexLast,
               NCollection_DataMap<int,occ::handle<MAT_BasicElt>>& newMap){
 for(int i=1;i<=length;i++){const NCollection_Sequence<int>& s=tool.Circuit()->RefToEqui(iline,i);int indFirst=s.Value(1);
  newMap.Bind(indexLast,graph->ChangeBasicElt(indFirst));indexLast++;
  for(int j=2;j<=s.Length();j++){bool m1=false,m2=false;int g1=0,g2=0,g3=0,g4=0;
   graph->FusionOfBasicElts(indFirst,s.Value(j),m1,g1,g2,m2,g3,g4);ga_flag(m1);ga_flag(m2);
   if(m1)tool.BisecFusion(g1,g2);if(m2)tool.BisecFusion(g3,g4);}}}
// `figure side join openresult`: the bisecting locus of the figure (side 0 left, 1 right; join
// 0 arc, 2 intersection), as BRepMAT2d_BisectingLocus::Compute computes it.
void tp_mat2d_locus_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFigureInput f=tp_read_figure(r);MAT_Side side=r.integer()==0?MAT_Left:MAT_Right;GeomAbs_JoinType join=tp_join(r.integer());bool open=r.flag();
  int nlines=f.figure.Length();NCollection_DataMap<MAT2d_BiInt,int> nbSect;tp_cut_sketch(f.figure,nbSect);
  for(int l=1;l<=nlines;l++){ga_int(f.figure.Value(l).Length());for(int k=1;;k++){if(!nbSect.IsBound(MAT2d_BiInt(l,k)))break;ga_int(nbSect(MAT2d_BiInt(l,k)));}}
  occ::handle<MAT2d_Circuit> circuit=new MAT2d_Circuit(join,open);circuit->Perform(f.figure,f.closed,1,side==MAT_Left);
  tp_circuit(circuit,nlines);
  MAT2d_Tool2d tool;tool.Sense(side);tool.SetJoinType(join);tool.InitItems(circuit);
  MAT2d_Mat2d mat(open);tp_muted([&]{if(open)mat.CreateMatOpen(tool);else mat.CreateMat(tool);});
  ga_flag(mat.IsDone());ga_flag(mat.SemiInfinite());ga_int(mat.NumberOfBisectors());
  if(mat.IsDone()){occ::handle<MAT_ListOfBisector> roots=new MAT_ListOfBisector();
   for(mat.Init();mat.More();mat.Next())roots->BackAdd(mat.Bisector());ga_int(roots->Number());
   occ::handle<MAT_Graph> graph=new MAT_Graph();graph->Perform(mat.SemiInfinite(),roots,tool.NumberOfItems(),mat.NumberOfBisectors());
   if(nlines>1){NCollection_DataMap<int,occ::handle<MAT_BasicElt>> newMap;int indexLast=1;
    for(int l=1;l<=nlines;l++)tp_fusion(graph,tool,l,tool.Circuit()->LineLength(l),indexLast,newMap);
    graph->ChangeBasicElts(newMap);graph->CompactArcs();graph->CompactNodes();}
   tp_graph(graph,tool);})
 ga_end();}
