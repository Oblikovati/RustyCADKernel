// The probe mesh data source, the twin of crates/tkmeshvs/examples/mv_probe/source.rs:
// 14 nodes and 8 elements (faces of 3, 4 and 5 nodes, two links, a prism, a 0D element and
// a triangle sharing a link edge), groups, nodal normals and advanced selection by variant.
// GetGeom zero-fills the whole caller buffer, so no native stale or uninitialised buffer
// value reaches the output.
static const double MV_NODES[14][3]={{0,0,0},{1,0,0},{1,1,0},{0,1,0},{2,0,.5},{2,1,.25},{.5,.5,1},
  {3,0,0},{3,1,0},{4,.5,0},{0,0,2},{1,0,2},{1,1,2.5},{5,5,5}};
struct MvElement{MeshVS_EntityType type;std::vector<int> nodes;};
static const MvElement MV_ELEMENTS[8]={{MeshVS_ET_Face,{1,2,3}},{MeshVS_ET_Face,{2,5,6,3}},
  {MeshVS_ET_Face,{5,8,10,9,6}},{MeshVS_ET_Link,{3,4}},{MeshVS_ET_Link,{2,3}},
  {MeshVS_ET_Volume,{1,2,3,11,12,13}},{MeshVS_ET_0D,{7}},{MeshVS_ET_Face,{1,3,4}}};
struct MvGroup{int id;MeshVS_EntityType type;std::vector<int> ids;};
static const MvGroup MV_GROUPS[5]={{201,MeshVS_ET_Node,{1,2,7,14}},{202,MeshVS_ET_Face,{101,102,103}},
  {203,MeshVS_ET_Link,{104,105}},{204,MeshVS_ET_Volume,{106}},{205,MeshVS_ET_0D,{107}}};
class MvSource:public MeshVS_DataSource3D{
public:
  int variant;TColStd_PackedMapOfInteger nodes,elements;
  MvSource(int theVariant,int nodeMask,int elemMask):variant(theVariant){
    for(int i=0;i<14;++i)if(nodeMask&(1<<i))nodes.Add(i+1);
    for(int i=0;i<8;++i)if(elemMask&(1<<i))elements.Add(101+i);}
  static bool fill(NCollection_Array1<double>& c,const std::vector<int>& ids){
    for(int i=c.Lower();i<=c.Upper();++i)c(i)=0.;
    if(c.Length()<3*int(ids.size()))return false;
    for(size_t k=0;k<ids.size();++k)for(int a=0;a<3;++a)c(c.Lower()+3*int(k)+a)=MV_NODES[ids[k]-1][a];
    return true;}
  bool GetGeom(const int id,const bool isElement,NCollection_Array1<double>& coords,int& nbNodes,MeshVS_EntityType& type) const override{
    if(!isElement){if(!nodes.Contains(id))return false;nbNodes=1;type=MeshVS_ET_Node;return fill(coords,{id});}
    if(!elements.Contains(id))return false;const MvElement& e=MV_ELEMENTS[id-101];
    nbNodes=int(e.nodes.size());type=e.type;return fill(coords,e.nodes);}
  bool GetGeomType(const int id,const bool isElement,MeshVS_EntityType& type) const override{
    if(!isElement){if(!nodes.Contains(id))return false;type=MeshVS_ET_Node;return true;}
    if(!elements.Contains(id))return false;type=MV_ELEMENTS[id-101].type;return true;}
  bool Get3DGeom(const int id,int& nbNodes,occ::handle<NCollection_HArray1<NCollection_Sequence<int>>>& data) const override{
    if(!elements.Contains(id)||MV_ELEMENTS[id-101].type!=MeshVS_ET_Volume)return false;
    nbNodes=6;data=GetPrismTopology(3);return true;}
  void* GetAddr(const int,const bool) const override{return nullptr;}
  bool GetNodesByElement(const int id,NCollection_Array1<int>& nodeIds,int& nbNodes) const override{
    if(!elements.Contains(id))return false;const MvElement& e=MV_ELEMENTS[id-101];
    if(nodeIds.Length()<int(e.nodes.size()))return false;nbNodes=int(e.nodes.size());
    for(int k=0;k<nbNodes;++k)nodeIds(nodeIds.Lower()+k)=e.nodes[k];return true;}
  const TColStd_PackedMapOfInteger& GetAllNodes() const override{return nodes;}
  const TColStd_PackedMapOfInteger& GetAllElements() const override{return elements;}
  bool GetNodeNormal(const int rank,const int id,double& nx,double& ny,double& nz) const override{
    if(!(variant&2)||!elements.Contains(id)||MV_ELEMENTS[id-101].type!=MeshVS_ET_Face)return false;
    const MvElement& e=MV_ELEMENTS[id-101];if(rank<1||rank>int(e.nodes.size()))return false;
    const double* p=MV_NODES[e.nodes[rank-1]-1];nx=.1*p[0];ny=.2*p[1];nz=1.+p[2];return true;}
  void GetAllGroups(TColStd_PackedMapOfInteger& ids) const override{
    if(!(variant&1)){MeshVS_DataSource::GetAllGroups(ids);return;}for(const MvGroup& g:MV_GROUPS)ids.Add(g.id);}
  bool GetGroup(const int id,MeshVS_EntityType& type,TColStd_PackedMapOfInteger& ids) const override{
    if(!(variant&1)||id<201||id>205)return MeshVS_DataSource::GetGroup(id,type,ids);
    const MvGroup& g=MV_GROUPS[id-201];type=g.type;for(int k:g.ids)ids.Add(k);return true;}
  bool IsAdvancedSelectionEnabled() const override{return (variant&4)!=0;}
};
static occ::handle<MvSource> mv_source(int variant,int nodeMask=0x3FFF,int elemMask=0xFF){return new MvSource(variant,nodeMask,elemMask);}
static void mv_map(const TColStd_PackedMapOfInteger& m){ga_int(m.Extent());for(TColStd_PackedMapOfInteger::Iterator it(m);it.More();it.Next())ga_int(it.Key());}
static void mv_hmap(const occ::handle<TColStd_HPackedMapOfInteger>& m){ga_flag(m.IsNull());if(!m.IsNull())mv_map(m->Map());}
static void mv_reals(const occ::handle<NCollection_HArray1<double>>& a){ga_flag(a.IsNull());if(a.IsNull())return;ga_int(a->Lower());ga_int(a->Length());for(int i=a->Lower();i<=a->Upper();++i)ga_real(a->Value(i));}
static void mv_topo(const occ::handle<NCollection_HArray1<NCollection_Sequence<int>>>& t){
  ga_flag(t.IsNull());if(t.IsNull())return;ga_int(t->Length());
  for(int i=t->Lower();i<=t->Upper();++i){ga_int(t->Value(i).Length());for(int v:t->Value(i))ga_int(v);}}
// `mv_ds Variant NodeMask ElemMask MaxNodes`: the default normal and bound members.
void mv_ds(std::istream& input){
  GaReader r{input};const int variant=r.integer(),nodeMask=r.integer(),elemMask=r.integer(),maxNodes=r.integer();
  occ::handle<MvSource> s=mv_source(variant,nodeMask,elemMask);
  for(int id=101;id<=108;++id){
    double n[3]={-7,-7,-7};GA_TRY(ga_flag(s->GetNormal(id,maxNodes,n[0],n[1],n[2]));for(double v:n)ga_real(v))
    for(bool nodal:{false,true}){occ::handle<NCollection_HArray1<double>> a;GA_TRY(ga_flag(s->GetNormalsByElement(id,nodal,maxNodes,a));mv_reals(a))}}
  Bnd_Box b=s->GetBoundingBox();ga_flag(b.IsVoid());if(!b.IsVoid()){double v[6];b.Get(v[0],v[1],v[2],v[3],v[4],v[5]);for(double x:v)ga_real(x);}
  TColStd_PackedMapOfInteger groups;s->GetAllGroups(groups);mv_map(groups);
  MeshVS_EntityType t=MeshVS_ET_All;TColStd_PackedMapOfInteger ids;ga_flag(s->GetGroup(203,t,ids));ga_int(int(t));mv_map(ids);
  ga_flag(s->GetGroupAddr(203)==nullptr);ga_flag(s->IsAdvancedSelectionEnabled());
  ga_end();}
// `mv_topology Kind BasePoints`: the static and cached prism (0) or pyramid (1) topologies.
void mv_topology(std::istream& input){
  GaReader r{input};const int kind=r.integer(),n=r.integer();occ::handle<MvSource> s=mv_source(0);
  mv_topo(kind==0?MeshVS_DataSource3D::CreatePrismTopology(n):MeshVS_DataSource3D::CreatePyramidTopology(n));
  auto first=kind==0?s->GetPrismTopology(n):s->GetPyramidTopology(n);auto second=kind==0?s->GetPrismTopology(n):s->GetPyramidTopology(n);
  ga_flag(first.IsNull());ga_flag(first==second);ga_end();}
// `mv_deformed Magnify VectorMask Magnify2`: the deformed geometry of every node and element.
void mv_deformed(std::istream& input){
  GaReader r{input};const double magnify=r.real();const int mask=r.integer();const double magnify2=r.real();
  occ::handle<MvSource> s=mv_source(0);occ::handle<MeshVS_DeformedDataSource> d=new MeshVS_DeformedDataSource(s,magnify);
  ga_real(d->GetMagnify());
  for(int i=1;i<=14;++i)if(mask&(1<<(i-1))){d->SetVector(i,gp_Vec(.1*i,-.2*i,.05*i*i));}
  d->SetVector(1,gp_Vec(1,2,3));if(mask&1)d->SetVector(1,gp_Vec(.1,-.2,.05));
  if(magnify2!=0)d->SetMagnify(magnify2);ga_real(d->GetMagnify());ga_int(d->GetVectors().Extent());
  NCollection_Array1<double> c(1,30);
  for(int pass=0;pass<2;++pass)for(int id=pass?101:1;id<=(pass?108:14);++id){
    int nb=-1;MeshVS_EntityType t=MeshVS_ET_All;GA_TRY(bool ok=d->GetGeom(id,pass==1,c,nb,t);ga_flag(ok);if(ok){ga_int(nb);ga_int(int(t));for(int k=1;k<=3*nb;++k)ga_real(c(k));})}
  MeshVS_EntityType t=MeshVS_ET_All;ga_flag(d->GetGeomType(106,true,t));ga_int(int(t));
  int nb=0;occ::handle<NCollection_HArray1<NCollection_Sequence<int>>> topo;ga_flag(d->Get3DGeom(106,nb,topo));ga_int(nb);mv_topo(topo);
  mv_map(d->GetAllNodes());mv_map(d->GetAllElements());ga_flag(d->GetNonDeformedDataSource()==s);
  d->SetNonDeformedDataSource(nullptr);mv_map(d->GetAllNodes());ga_flag(d->GetGeom(1,false,c,nb,t));
  ga_end();}
