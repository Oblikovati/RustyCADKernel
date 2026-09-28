// Poly_CoherentTriangulation, Poly_MergeNodesTool and Poly_MakeLoops oracle at pinned OCCT 3d097a0.
#include <Poly_CoherentTriangulation.hxx>
#include <Poly_MakeLoops.hxx>
#include <Poly_MergeNodesTool.hxx>
#include <gp_Dir2d.hxx>
#include <map>
#include <vector>
namespace poly_tools_probe {
using poly_probe::failure;using poly_probe::integer;using poly_probe::real;
// Protected arrays are read to turn triangle and link pointers into indices.
struct Coherent : public Poly_CoherentTriangulation {
 explicit Coherent(const occ::handle<Poly_Triangulation>& t):Poly_CoherentTriangulation(t){}
 int tri(const Poly_CoherentTriangle* p)const{if(!p)return -1;for(int i=0;i<myTriangles.Length();++i)if(&myTriangles(i)==p)return i;return -2;}
 int link(const Poly_CoherentLink* p)const{if(!p)return -1;for(int i=0;i<myLinks.Length();++i)if(&myLinks(i)==p)return i;return -2;}
 Poly_CoherentTriangle& triangle(int i){return myTriangles(i);}
 Poly_CoherentLink& linkAt(int i){return myLinks(i);}
 int links()const{return myLinks.Length();}
};
void state(const occ::handle<Coherent>& c){
 std::cout<<c->MaxNode()<<' '<<c->MaxTriangle()<<' '<<c->NNodes()<<' '<<c->NTriangles()<<' '<<c->NLinks()<<" T ";
 for(int i=0;i<=c->MaxTriangle();++i){const Poly_CoherentTriangle& t=c->Triangle(i);
  std::cout<<t.Node(0)<<' '<<t.Node(1)<<' '<<t.Node(2)<<' '<<t.NConnections()<<' ';
  for(int k=0;k<3;++k)std::cout<<c->tri(t.GetConnectedTri(k))<<' '<<t.GetConnectedNode(k)<<' '<<c->link(t.GetLink(k))<<' ';}
 std::cout<<"L ";for(int i=0;i<c->links();++i){const Poly_CoherentLink& l=c->linkAt(i);std::cout<<l.Node(0)<<' '<<l.Node(1)<<' '<<l.OppositeNode(0)<<' '<<l.OppositeNode(1)<<' ';}
 std::cout<<"N ";for(int i=0;i<=c->MaxNode();++i){const Poly_CoherentNode& n=c->Node(i);std::cout<<n.IsFreeNode()<<' '<<n.GetIndex()<<' ';
  if(!n.IsFreeNode())for(Poly_CoherentTriPtr::Iterator it=n.TriangleIterator();it.More();it.Next())std::cout<<c->tri(&it.Value())<<' ';std::cout<<"; ";}
}
void edit(const occ::handle<Coherent>& c,std::istream& in){
 int op=integer(in),a=integer(in),b=integer(in),d=integer(in),e=integer(in);std::cout<<"E"<<op<<' ';
 if(op==1)std::cout<<c->RemoveTriangle(c->triangle(a))<<' ';
 else if(op==2)std::cout<<c->tri(c->AddTriangle(a,b,d))<<' ';
 else if(op==3)std::cout<<c->ReplaceNodes(c->triangle(a),b,d,e)<<' ';
 else if(op==4){if(a<c->links())c->RemoveLink(c->linkAt(a));}
 else if(op==5)std::cout<<c->ComputeLinks()<<' ';
 else if(op==6)c->ClearLinks();
 else if(op==7)std::cout<<c->SetNode(gp_XYZ(a,b,d),-1)<<' ';
 else if(op==8)std::cout<<c->link(c->AddLink(c->triangle(a),b))<<' ';
}
void found(const occ::handle<Coherent>& c){
 for(Poly_CoherentTriangulation::IteratorOfLink it(c);it.More();it.Next()){const Poly_CoherentTriangle* t[2];
  bool ok=c->FindTriangle(it.Value(),t);std::cout<<ok<<' '<<c->tri(t[0])<<' '<<c->tri(t[1])<<' ';}
}
void exported(const occ::handle<Poly_Triangulation>& r){
 if(r.IsNull()){std::cout<<"null ";return;}
 std::cout<<r->NbNodes()<<' '<<r->NbTriangles()<<' '<<r->HasNormals()<<' '<<r->HasUVNodes()<<' '<<bits(r->Deflection())<<' ';
 for(int i=1;i<=r->NbNodes();++i){xyz(r->Node(i).XYZ());if(r->HasNormals()){NCollection_Vec3<float> v;r->Normal(i,v);poly_probe::fvec(v);}if(r->HasUVNodes())xy(r->UVNode(i).XY());}
 for(int i=1;i<=r->NbTriangles();++i){int a,b,d;r->Triangle(i).Get(a,b,d);std::cout<<a<<' '<<b<<' '<<d<<' ';}
}
void coherent_case(std::istream& in){
 occ::handle<Poly_Triangulation> m=poly_probe::mesh(in);int flags=integer(in);double tol=real(in);int edits=integer(in);
 m->Deflection(tol*3);if(flags&1)m->ComputeNormals();
 if(flags&2){m->AddUVNodes();for(int i=1;i<=m->NbNodes();++i)m->SetUVNode(i,gp_Pnt2d(m->Node(i).Z(),i));}
 try{occ::handle<Coherent> c=new Coherent(m);std::cout<<bits(c->Deflection())<<' ';state(c);
  if(flags&4){std::cout<<c->ComputeLinks()<<' ';state(c);found(c);}
  for(int i=0;i<edits;++i){edit(c,in);state(c);}
  NCollection_List<Poly_CoherentTriangulation::TwoIntegers> removed;
  std::cout<<c->RemoveDegenerated(tol,&removed)<<' ';for(const auto& p:removed)std::cout<<p.myValue[0]<<' '<<p.myValue[1]<<' ';state(c);
  NCollection_List<int> free;std::cout<<c->GetFreeNodes(free)<<' ';for(int n:free)std::cout<<n<<' ';
  exported(c->GetTriangulation());
  occ::handle<Poly_CoherentTriangulation> copy=c->Clone(occ::handle<NCollection_BaseAllocator>());
  if(copy.IsNull())std::cout<<"null ";else std::cout<<copy->NNodes()<<' '<<copy->NTriangles()<<' '<<bits(copy->Deflection())<<' ';
  std::ostringstream dump;c->Dump(dump);std::cout<<poly_probe::flat(dump.str())<<' ';
 }catch(const Standard_Failure& f){failure(f);}
 std::cout<<'\n';
}
void merged(const occ::handle<Poly_Triangulation>& r){
 if(r.IsNull()){std::cout<<"null ";return;}
 std::cout<<r->NbNodes()<<' '<<r->NbTriangles()<<' '<<r->IsDoublePrecision()<<' ';
 for(int i=1;i<=r->NbNodes();++i)xyz(r->Node(i).XYZ());
 for(int i=1;i<=r->NbTriangles();++i){int a,b,d;r->Triangle(i).Get(a,b,d);std::cout<<a<<' '<<b<<' '<<d<<' ';}
}
void counters(const Poly_MergeNodesTool& t){
 std::cout<<t.NbNodes()<<' '<<t.NbElements()<<' '<<t.NbDegenerativeElems()<<' '<<t.NbMergedElems()<<' ';
 for(int i=0;i<4;++i)std::cout<<t.ElementNodeIndex(i)<<' ';std::cout<<bits(t.MergeAngle())<<' '<<bits(t.MergeTolerance())<<' ';
}
void merge_case(std::istream& in){
 occ::handle<Poly_Triangulation> m=poly_probe::mesh(in);gp_Trsf tr=poly_probe::transform(in);
 bool reverse=integer(in)!=0;double angle=real(in),tol=real(in),unit=real(in);int flags=integer(in),facets=integer(in);
 try{merged(Poly_MergeNodesTool::MergeNodes(m,tr,reverse,angle,tol,(flags&8)!=0));}catch(const Standard_Failure& f){failure(f);}
 try{Poly_MergeNodesTool tool(angle,tol,facets);tool.SetMergeOpposite(flags&1);tool.SetMergeElems(flags&2);tool.SetDropDegenerative(!(flags&4));
  tool.SetUnitFactor(unit);std::cout<<tool.ToMergeOpposite()<<tool.ToMergeElems()<<tool.ToDropDegenerative()<<' ';
  tool.AddTriangulation(m,tr,reverse);counters(tool);
  if(m->NbNodes()>=4){gp_XYZ q[4]={m->Node(1).XYZ(),m->Node(2).XYZ(),m->Node(3).XYZ(),m->Node(4).XYZ()};tool.AddQuad(q);counters(tool);}
  gp_XYZ t3[3]={m->Node(1).XYZ(),m->Node(2).XYZ(),m->Node(3).XYZ()};tool.AddTriangle(t3);counters(tool);
  tool.ChangeElementNode(0)=m->Node(1).XYZ();tool.ChangeElementNode(1)=m->Node(1).XYZ();tool.ChangeElementNode(2)=m->Node(2).XYZ();
  tool.PushLastTriangle();counters(tool);
  try{tool.PushLastElement(5);}catch(const Standard_Failure& f){failure(f);}
  std::cout<<bits(tool.computeTriNormal().x())<<' ';merged(tool.Result());
 }catch(const Standard_Failure& f){failure(f);}
 try{Poly_MergeNodesTool map(angle,tol);map.ChangeOutput().Nullify();map.AddTriangulation(m,tr,reverse);
  gp_XYZ q[4]={m->Node(1).XYZ(),m->Node(2).XYZ(),m->Node(3).XYZ(),m->Node(1).XYZ()};map.AddQuad(q);counters(map);std::cout<<map.Result().IsNull()<<' ';
 }catch(const Standard_Failure& f){failure(f);}
 std::cout<<'\n';
}
// Straight links: the tangent at either end is the chord of the oriented link.
struct LoopHelper : public Poly_MakeLoops3D::Helper, public Poly_MakeLoops2D::Helper {
 std::vector<gp_XY> points;std::map<int,Poly_MakeLoops::ListOfLink> adjacent;Poly_MakeLoops::ListOfLink none;
 const Poly_MakeLoops::ListOfLink& GetAdjacentLinks(int n)const override{auto f=adjacent.find(n);return f==adjacent.end()?none:f->second;}
 gp_XY chord(const Poly_MakeLoops::Link& l)const{gp_XY d=points[l.node2-1]-points[l.node1-1];return l.IsReversed()?-d:d;}
 bool GetFirstTangent(const Poly_MakeLoops::Link& l,gp_Dir& d)const override{gp_XY c=chord(l);if(c.Modulus()<1e-9)return false;d=gp_Dir(c.X(),c.Y(),0);return true;}
 bool GetLastTangent(const Poly_MakeLoops::Link& l,gp_Dir& d)const override{return GetFirstTangent(l,d);}
 bool GetNormal(int n,gp_Dir& d)const override{if(n%5==4)return false;d=gp_Dir(0,0,1);return true;}
 bool GetFirstTangent(const Poly_MakeLoops::Link& l,gp_Dir2d& d)const override{gp_XY c=chord(l);if(c.Modulus()<1e-9)return false;d=gp_Dir2d(c);return true;}
 bool GetLastTangent(const Poly_MakeLoops::Link& l,gp_Dir2d& d)const override{return GetFirstTangent(l,d);}
};
void loop_link(const Poly_MakeLoops::Link& l){std::cout<<l.node1<<' '<<l.node2<<' '<<l.flags<<' ';}
// The helper must list replaced links with their new node order, as the algorithm reads both.
void swap_adjacent(LoopHelper& h,const Poly_MakeLoops::Link& old,const Poly_MakeLoops::Link& now){
 for(auto& entry:h.adjacent)for(auto& l:entry.second)if(l.node1==old.node1&&l.node2==old.node2){l.node1=now.node1;l.node2=now.node2;}
}
void loops_run(Poly_MakeLoops& algo,LoopHelper& h,const std::vector<Poly_MakeLoops::Link>& links,std::istream& in){
 for(const auto& l:links)algo.AddLink(l);
 int k=integer(in),orient=integer(in);if(k<int(links.size()))std::cout<<algo.SetLinkOrientation(links[k],Poly_MakeLoops::LinkFlag(orient))<<' ';
 int r=integer(in);if(r<int(links.size())){Poly_MakeLoops::Link n(links[r].node2,links[r].node1);n.flags=links[r].flags;algo.ReplaceLink(links[r],n);swap_adjacent(h,links[r],n);}
 loop_link(algo.FindLink(links[0]));std::cout<<algo.Perform()<<' '<<algo.GetNbLoops()<<' ';
 for(int i=1;i<=algo.GetNbLoops();++i){std::cout<<'[';for(const auto& l:algo.GetLoop(i))loop_link(l);std::cout<<"] ";}
 Poly_MakeLoops::ListOfLink hanging;algo.GetHangingLinks(hanging);std::cout<<algo.GetNbHanging()<<' ';for(const auto& l:hanging)loop_link(l);
}
void loops_case(std::istream& in){
 int variant=integer(in),n=integer(in);LoopHelper h;for(int i=0;i<n;++i){double x=real(in),y=real(in);h.points.emplace_back(x,y);}
 int m=integer(in);std::vector<Poly_MakeLoops::Link> links;
 for(int i=0;i<m;++i){int a=integer(in),b=integer(in);Poly_MakeLoops::Link l(a,b);l.flags=integer(in);links.push_back(l);
  h.adjacent[l.node1].Append(l);if(l.node2!=l.node1)h.adjacent[l.node2].Append(l);}
 if(variant==2){Poly_MakeLoops3D algo(&h,occ::handle<NCollection_BaseAllocator>());loops_run(algo,h,links,in);}
 else{Poly_MakeLoops2D algo(variant==0,&h,occ::handle<NCollection_BaseAllocator>());loops_run(algo,h,links,in);}
 std::cout<<'\n';
}
} // namespace poly_tools_probe
