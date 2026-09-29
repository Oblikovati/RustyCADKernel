// BRepGraph oracle, part 4: BRepGraph_ChildExplorer and BRepGraph_ParentExplorer from
// every node (the child explorer also past each kind's end) under a set of configurations,
// printing every emitted instance with its path queries. The Rust twin is
// crates/tkbrep/examples/br_probe/brep_graph_explore.rs.
#include <BRepGraph_ChildExplorer.hxx>
#include <BRepGraph_ParentExplorer.hxx>
void bx_instance(const BrScene& s,const BRepGraphInc::NodeInstance& i){bg_node(i.DefId);br_chain(s.graph,i.Location);br_int(i.Orientation);}
void bx_child(const BrScene& s,BRepGraph_ChildExplorer& x){
 const BRepGraph_ChildExplorer::Config& c=x.GetConfig();br_flag(c.AvoidKind.has_value());if(c.AvoidKind)br_int(int(*c.AvoidKind));
 for(;x.More();x.Next()){
  bx_instance(s,x.Current());bg_node(x.CurrentParent());br_int(int(x.CurrentLinkKind()));bg_ref(x.CurrentRef());
  BRepGraph_UsagePath p=x.CurrentUsagePath();br_int(int(p.Size()));
  for(size_t i=0;i<p.Size();++i){bg_node(p.Value(i).Node);bg_ref(p.Value(i).Ref);br_int(p.Value(i).StepIndex);}
  br_int(x.Depth());br_chain(s.graph,x.LocationOf(BRepGraph_NodeId::Kind::Face));bg_node(x.NodeOf(BRepGraph_NodeId::Kind::Edge));
  for(int level=-1;level<3;++level){br_chain(s.graph,x.LocationAt(level));bg_node(x.NodeAt(level));}}
 bx_instance(s,x.Current());br_word("|");}
// The child configurations: 0 all, 1 direct, 2-10 a target recursive, 11-13 a target
// direct, 14-16 avoid kinds, 17 no accumulation, 18 an explicit start transform.
void bx_child_configs(const BrScene& s,const BRepGraph& g,const BRepGraph_NodeId& n,const TopLoc_Location& l){
 using X=BRepGraph_ChildExplorer;using K=BRepGraph_NodeId::Kind;
 const K targets[9]={K::Solid,K::Shell,K::Face,K::Wire,K::CoEdge,K::Edge,K::Vertex,K::Occurrence,K::Product};
 try{X x(g,n);bx_child(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 try{X x(g,n,X::TraversalMode::DirectChildren);bx_child(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 for(K k:targets){try{X x(g,n,k);bx_child(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
 for(K k:{K::Face,K::Edge,K::Occurrence}){try{X x(g,n,k,X::TraversalMode::DirectChildren);bx_child(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
 try{X x(g,n,std::optional<K>(K::Face),true);bx_child(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 try{X x(g,n,std::optional<K>(K::Wire),false);bx_child(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 try{X x(g,n,K::Vertex,std::optional<K>(K::Face),true);bx_child(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 try{X x(g,n,K::Vertex,false,false);bx_child(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 try{X x(g,n,K::Edge,l,TopAbs_REVERSED);bx_child(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
void bx_parent(const BrScene& s,BRepGraph_ParentExplorer& x){
 const BRepGraph_ParentExplorer::Config& c=x.GetConfig();br_flag(c.AvoidKind.has_value());if(c.AvoidKind)br_int(int(*c.AvoidKind));
 for(;x.More();x.Next()){
  bx_instance(s,x.Current());bg_node(x.CurrentChild());br_int(int(x.CurrentLinkKind()));bg_ref(x.CurrentRef());
  br_chain(s.graph,x.LeafLocation());br_int(x.LeafOrientation());br_flag(x.IsCurrentBranchRoot());}
 bx_instance(s,x.Current());br_word("|");}
// The parent configurations: 0 all, 1 direct, 2-8 a target recursive, 9-10 a target
// direct, 11-13 avoid kinds.
void bx_parent_configs(const BrScene& s,const BRepGraph& g,const BRepGraph_NodeId& n){
 using X=BRepGraph_ParentExplorer;using K=BRepGraph_NodeId::Kind;
 const K targets[7]={K::Edge,K::Wire,K::Face,K::Shell,K::Solid,K::Compound,K::Product};
 try{X x(g,n);bx_parent(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 try{X x(g,n,X::TraversalMode::DirectParents);bx_parent(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 for(K k:targets){try{X x(g,n,k);bx_parent(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
 for(K k:{K::Face,K::Occurrence}){try{X x(g,n,k,X::TraversalMode::DirectParents);bx_parent(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
 try{X x(g,n,std::optional<K>(K::Shell),true);bx_parent(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 try{X x(g,n,std::optional<K>(K::Face),false);bx_parent(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}
 try{X x(g,n,K::Solid,std::optional<K>(K::Face),true);bx_parent(s,x);}catch(const Standard_Failure& e){br_word(e.ExceptionType());}}
void bx_explore(const BrScene& s,const BRepGraph& g){
 const TopLoc_Location& l=s.graph.loc(int(s.graph.locs.size())-1);
 for(const BRepGraph_NodeId& n:bw_nodes(g)){bx_child_configs(s,g,n,l);bx_parent_configs(s,g,n);}
 // Past each kind's end only the child explorer (the parent explorer reads the
 // relations of its start node unchecked natively).
 for(int k=0;k<12;++k)bx_child_configs(s,g,BRepGraph_NodeId(BRepGraph_NodeId::Kind(k),g.Topo().Gen().Nb(BRepGraph_NodeId::Kind(k))),l);
 br_word("#");}
