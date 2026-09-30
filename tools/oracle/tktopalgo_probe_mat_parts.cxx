// MAT and MAT2d probe operations on the classes one by one; the Rust twin is
// crates/tktopalgo/examples/tp_probe/mat_parts.rs. It uses the figure reader of
// tktopalgo_probe_mat.cxx.
// The list of edges: `n` edges numbered 1..n, then the edits.
int tp_edge_number(const occ::handle<MAT_Edge>& e){return e.IsNull()?-1:e->EdgeNumber();}
// The members that dereference a node are read only where the node exists (natively a null
// node crashes): the current item when there is one, the neighbours on a looped list.
void tp_list_state(const occ::handle<MAT_ListOfEdge>& l,bool looped){
 ga_int(l->Number());ga_int(l->Index());ga_flag(l->More());ga_flag(l->IsEmpty());
 if(l->More())ga_int(tp_edge_number(l->Current()));
 if(!l->IsEmpty()){ga_int(tp_edge_number(l->FirstItem()));ga_int(tp_edge_number(l->LastItem()));}
 if(looped&&l->More()){ga_int(tp_edge_number(l->PreviousItem()));ga_int(tp_edge_number(l->NextItem()));}}
occ::handle<MAT_Edge> tp_new_edge(int number){occ::handle<MAT_Edge> e=new MAT_Edge();e->EdgeNumber(number);return e;}
// `n (op arg)*`: op 0 First, 1 Last, 2 Next, 3 Previous, 4 Brackets(arg), 5 Unlink,
// 6 LinkBefore(new edge arg), 7 LinkAfter(new edge arg), 8 FrontAdd(new edge arg),
// 9 Permute, 10 Loop, 11 Init(edge arg of the first n), 12 Current(new edge arg),
// 13 BackAdd(new edge arg); the state after each.
void tp_mat_list_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int n=r.integer();occ::handle<MAT_ListOfEdge> l=new MAT_ListOfEdge();std::vector<occ::handle<MAT_Edge>> edges;
  for(int i=1;i<=n;i++){edges.push_back(tp_new_edge(i));l->BackAdd(edges.back());}
  bool looped=false;tp_list_state(l,looped);int ops=r.integer();
  for(int k=0;k<ops;k++){int op=r.integer(),arg=r.integer();
   GA_TRY(switch(op){case 0:l->First();break;case 1:l->Last();break;case 2:l->Next();break;case 3:l->Previous();break;
    case 4:ga_int(tp_edge_number(l->Brackets(arg)));break;case 5:l->Unlink();break;case 6:l->LinkBefore(tp_new_edge(arg));break;
    case 7:l->LinkAfter(tp_new_edge(arg));break;case 8:l->FrontAdd(tp_new_edge(arg));break;case 9:l->Permute();break;
    case 10:l->Loop();looped=true;break;case 11:l->Init(edges[arg-1]);break;case 12:l->Current(tp_new_edge(arg));break;default:l->BackAdd(tp_new_edge(arg));})
   tp_list_state(l,looped);}
  ga_text(l->DynamicType()->Name()))
 ga_end();}
// `shift level`: MAT_Bisector with a list of two sons, then its Dump and the Dump of its list.
void tp_mat_bisector_case(std::istream& input){
 GaReader r{input};
 GA_TRY(int shift=r.integer(),level=r.integer();occ::handle<MAT_Bisector> b=new MAT_Bisector();
  b->FirstEdge(tp_new_edge(1));b->SecondEdge(tp_new_edge(2));b->BisectorNumber(7);
  for(int i=0;i<2;i++){occ::handle<MAT_Bisector> s=new MAT_Bisector();s->FirstEdge(tp_new_edge(3+i));s->SecondEdge(tp_new_edge(5+i));s->BisectorNumber(8+i);b->AddBisector(s);}
  ga_int(b->BisectorNumber());ga_int(b->IndexNumber());ga_real(b->FirstParameter());ga_real(b->SecondParameter());ga_real(b->Sense());
  ga_int(b->FirstBisector()->BisectorNumber());ga_int(b->LastBisector()->BisectorNumber());
  ga_captured([&]{b->Dump(shift,level);});b->List()->First();ga_captured([&]{b->Dump(shift,level);});
  ga_captured([&]{b->List()->Dump(shift,level);});
  ga_text(b->DynamicType()->Name());ga_text(b->List()->DynamicType()->Name()))
 ga_end();}
// `-`: a hand-made MAT graph piece: arcs, nodes and basic elements, their links and errors.
void tp_mat_arc_case(std::istream& input){
 GaReader r{input};(void)r;
 GA_TRY(occ::handle<MAT_BasicElt> e1=new MAT_BasicElt(1),e2=new MAT_BasicElt(2),e3=new MAT_BasicElt(3);
  occ::handle<MAT_Arc> a1=new MAT_Arc(1,11,e1,e2),a2=new MAT_Arc(2,12,e2,e3),a3=new MAT_Arc(3,13,e3,e1);
  occ::handle<MAT_Node> n=new MAT_Node(5,a1,2.5),m=new MAT_Node(6,a1,0.0),o=new MAT_Node(7,a2,Precision::Infinite());
  a1->SetFirstNode(n);a2->SetFirstNode(n);a3->SetFirstNode(n);a1->SetSecondNode(m);a2->SetSecondNode(o);
  a1->SetNeighbour(MAT_Left,n,a2);a2->SetNeighbour(MAT_Left,n,a3);a3->SetNeighbour(MAT_Left,n,a1);
  a1->SetNeighbour(MAT_Right,n,a3);a2->SetFirstArc(MAT_Right,a1);a3->SetSecondArc(MAT_Left,a2);
  e1->SetStartArc(a1);e1->SetEndArc(a3);e1->SetGeomIndex(4);e1->SetIndex(9);n->SetIndex(3);a3->SetGeomIndex(14);a3->SetIndex(4);
  ga_int(e1->Index());ga_int(e1->GeomIndex());ga_int(tp_index(e1->StartArc()));ga_int(tp_index(e1->EndArc()));
  ga_int(a3->Index());ga_int(a3->GeomIndex());ga_int(tp_index(a1->FirstElement()));ga_int(tp_index(a1->SecondElement()));
  ga_int(tp_index(a1->TheOtherNode(n)));ga_int(tp_index(a1->TheOtherNode(m)));GA_TRY(a1->TheOtherNode(o))
  for(const occ::handle<MAT_Node>& x:{n,m,o})for(MAT_Side s:{MAT_Left,MAT_Right}){GA_TRY(ga_flag(a1->HasNeighbour(x,s)))GA_TRY(ga_int(tp_index(a1->Neighbour(x,s))))}
  GA_TRY(a1->SetNeighbour(MAT_Left,o,a2))
  ga_int(n->Index());ga_int(n->GeomIndex());ga_real(n->Distance());ga_flag(n->PendingNode());ga_flag(m->PendingNode());
  ga_flag(n->OnBasicElt());ga_flag(m->OnBasicElt());ga_flag(o->Infinite());
  NCollection_Sequence<occ::handle<MAT_Arc>> arcs;n->LinkedArcs(arcs);ga_int(arcs.Length());for(const occ::handle<MAT_Arc>& a:arcs)ga_int(tp_index(a));
  NCollection_Sequence<occ::handle<MAT_BasicElt>> elts;n->NearElts(elts);ga_int(elts.Length());for(const occ::handle<MAT_BasicElt>& e:elts)ga_int(tp_index(e));
  m->SetLinkedArc(a2);GA_TRY(ga_flag(m->PendingNode()))
  occ::handle<MAT_Zone> z=new MAT_Zone();ga_flag(z->NoEmptyZone());ga_flag(z->Limited());GA_TRY(z->ArcOnFrontier(1))
  occ::handle<MAT_Zone> z2=new MAT_Zone(e2);ga_int(z2->NumberOfArcs());ga_flag(z2->Limited());
  occ::handle<MAT_Graph> g=new MAT_Graph();ga_int(g->NumberOfArcs());GA_TRY(g->Arc(1))GA_TRY(g->Node(1))GA_TRY(g->BasicElt(1))
  ga_text(a1->DynamicType()->Name());ga_text(n->DynamicType()->Name());ga_text(e1->DynamicType()->Name());ga_text(z->DynamicType()->Name());
  ga_text(g->DynamicType()->Name()))
 ga_end();}
// `l1 l2 i1 i2 d u1 u2 x1 y1 x2 y2` twice, then a sense: MAT2d_Connexion.
occ::handle<MAT2d_Connexion> tp_read_connexion(GaReader& r){
 int l1=r.integer(),l2=r.integer(),i1=r.integer(),i2=r.integer();double d=r.real(),u1=r.real(),u2=r.real();gp_Pnt2d p1=r.pnt2d();gp_Pnt2d p2=r.pnt2d();
 return new MAT2d_Connexion(l1,l2,i1,i2,d,u1,u2,p1,p2);}
void tp_mat2d_connexion_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<MAT2d_Connexion> a=tp_read_connexion(r),b=tp_read_connexion(r);double sense=r.real();
  ga_flag(a->IsAfter(b,sense));ga_flag(b->IsAfter(a,sense));ga_flag(a->IsAfter(a,sense));occ::handle<MAT2d_Connexion> c=a->Reverse();tp_connexion(c);
  ga_captured([&]{a->Dump(0,1);});occ::handle<MAT2d_Connexion> e=new MAT2d_Connexion();tp_connexion(e);
  e->IndexFirstLine(3);e->IndexSecondLine(4);e->IndexItemOnFirst(5);e->IndexItemOnSecond(6);e->ParameterOnFirst(0.5);e->ParameterOnSecond(0.25);
  e->PointOnFirst(gp_Pnt2d(1.,2.));e->PointOnSecond(gp_Pnt2d(3.,4.));e->Distance(7.);tp_connexion(e);
  MAT2d_BiInt k(1,2);k.FirstIndex(3);k.SecondIndex(4);ga_int(k.FirstIndex());ga_int(k.SecondIndex());ga_flag(k.IsEqual(MAT2d_BiInt(3,4)));
  ga_flag(k==MAT2d_BiInt(4,3));ga_text(a->DynamicType()->Name()))
 ga_end();}
// `curve`: MAT2d_CutCurve.
void tp_mat2d_cut_case(std::istream& input){
 GaReader r{input};
 GA_TRY(occ::handle<Geom2d_Curve> c=tp_curve2d(r);MAT2d_CutCurve cut(c);ga_flag(cut.UnModified());
  GA_TRY(int n=cut.NbCurves();ga_int(n);for(int i=1;i<=n;i++){occ::handle<Geom2d_TrimmedCurve> t=cut.Value(i);ga_real(t->FirstParameter());ga_real(t->LastParameter());})
  GA_TRY(cut.Value(0))MAT2d_CutCurve empty;ga_flag(empty.UnModified());GA_TRY(empty.Value(1)))
 ga_end();}
// `figure indstart sense`: MAT2d_MiniPath on the figure (lines of point or curve items).
void tp_mat2d_minipath_case(std::istream& input){
 GaReader r{input};
 GA_TRY(TpFigureInput f=tp_read_figure(r);int ind=r.integer();bool sense=r.flag();MAT2d_MiniPath path;path.Perform(f.figure,ind,sense);
  ga_int(path.Path().Length());for(const occ::handle<MAT2d_Connexion>& c:path.Path())tp_connexion(c);
  for(int l=1;l<=f.figure.Length();l++){ga_flag(path.IsRoot(l));ga_flag(path.IsConnexionsFrom(l));
   if(path.IsConnexionsFrom(l))ga_int(path.ConnexionsFrom(l).Length());GA_TRY(tp_connexion(path.Father(l)))}
  GA_TRY(path.ConnexionsFrom(f.figure.Length()+1))MAT2d_MiniPath empty;GA_TRY(empty.RunOnConnexions()))
 ga_end();}
// `tool`: MAT2d_Tool2d without circuit (the members that do not need one) and Dump.
void tp_mat2d_tool_case(std::istream& input){
 GaReader r{input};(void)r;
 GA_TRY(MAT2d_Tool2d tool;ga_real(tool.ToleranceOfConfusion());GA_TRY(tool.GeomPnt(1))GA_TRY(tool.GeomVec(1))GA_TRY(tool.GeomBis(1))
  GA_TRY(tool.Dump(1,0))MAT2d_Mat2d mat;ga_flag(mat.IsDone());ga_flag(mat.SemiInfinite());ga_int(mat.NumberOfBisectors()))
 ga_end();}
