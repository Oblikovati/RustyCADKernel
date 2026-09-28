// BVH phase 2 oracle at pinned OCCT 3d097a0: linear, spatial-median, sweep-plane and
// two-bin builds (tree layouts), box sets, single/pair traversal logs, BVH_Distance,
// BVH_PairDistance, the distance-field point queries and BVH_DistanceField grids.
// Included by tkmath_probe.cxx after tkmath_bvh_probe.cxx (reuses its helpers).
#include <BVH_LinearBuilder.hxx>
#include <BVH_SpatialMedianBuilder.hxx>
#include <BVH_SweepPlaneBuilder.hxx>
#include <BVH_BoxSet.hxx>
#include <BVH_IndexedBoxSet.hxx>
#include <BVH_Traverse.hxx>
#include <BVH_Distance.hxx>
#include <BVH_PairDistance.hxx>
#include <BVH_Geometry.hxx>
#include <BVH_DistanceField.hxx>
#include <type_traits>
// kind: 0 linear (flag = parallel), 1 spatial median (flag = main axis), 2 sweep plane,
// 3 binned with two bins (flag = main axis), otherwise binned with 48 bins.
template<class T,int N> opencascade::handle<BVH_Builder<T,N>> bvh_query_builder(int kind,int leaf,int depth,int flag) {
 if(kind==0){BVH_LinearBuilder<T,N>* linear=new BVH_LinearBuilder<T,N>(leaf,depth);linear->SetParallel(flag!=0);return linear;}
 if(kind==1)return new BVH_SpatialMedianBuilder<T,N>(leaf,depth,flag!=0);
 if(kind==2)return new BVH_SweepPlaneBuilder<T,N>(leaf,depth,1);
 if(kind==3)return new BVH_BinnedBuilder<T,N,2>(leaf,depth,flag!=0);
 return new BVH_BinnedBuilder<T,N,48>(leaf,depth,flag!=0);
}
// mode: 0 set box after MarkDirty, 1 invalid box, 2 point box at the first vertex, 3 widened box.
template<class T,int N,class Set> BVH_Box<T,N> bvh_query_box(Set& set,int mode,const BVH_Triangulation<T,N>& mesh) {
 if(mode==1)return BVH_Box<T,N>();
 if(mode==2)return mesh.Vertices.Size()>0?BVH_Box<T,N>(mesh.Vertices.Value(0)):BVH_Box<T,N>();
 set.MarkDirty();BVH_Box<T,N> box=set.Box();
 if(mode==3&&box.IsValid())for(int i=0;i<N;++i){box.CornerMin()[i]-=T(1);box.CornerMax()[i]+=T(2);}
 return box;
}
template<class T,int N> void bvh_query_order(const BVH_Triangulation<T,N>& set) {bvh_order_out(set);}
template<class Set> void bvh_query_order(const Set& set) {for(int i=0;i<set.Size();++i)std::cout<<set.Element(i)<<' ';}
// Explicit build, empty rebuild into the same tree, then the set's own BVH().
template<class T,int N,class Set> void bvh_builders_on(Set& set,const opencascade::handle<BVH_Builder<T,N>>& builder,int mode,const BVH_Triangulation<T,N>& mesh) {
 BVH_Box<T,N> box=bvh_query_box<T,N>(set,mode,mesh);bvh_box_out(box);
 BVH_Tree<T,N> tree;builder->Build(&set,&tree,box);bvh_nodes_out(tree,N);bvh_query_order(set);
 if(tree.Length()>0)bvh_bits(tree.EstimateSAH());
 Set empty(builder);builder->Build(&empty,&tree,box);std::cout<<tree.Length()<<' '<<tree.Depth()<<' ';
 set.MarkDirty();bvh_nodes_out(*set.BVH(),N);bvh_query_order(set);std::cout<<'\n';
}
// input: kind leaf depth flag setkind(0 mesh, 1 box set, 2 indexed box set) boxmode count, vertices.
template<class T,int N> void bvh_builders_run(std::istream& input) {
 int kind,leaf,depth,flag,setkind,mode,count;input>>kind>>leaf>>depth>>flag>>setkind>>mode>>count;
 opencascade::handle<BVH_Builder<T,N>> builder=bvh_query_builder<T,N>(kind,leaf,depth,flag);
 BVH_Triangulation<T,N> mesh(builder);bvh_fill(mesh,input,count);
 if(setkind==0){bvh_builders_on<T,N>(mesh,builder,mode,mesh);return;}
 if constexpr(std::is_same<T,double>::value){
  if(setkind==1){BVH_BoxSet<T,N> boxes(builder);for(int i=0;i<count;++i)boxes.Add(i,mesh.Box(i));bvh_builders_on<T,N>(boxes,builder,mode,mesh);return;}
  BVH_IndexedBoxSet<T,N> boxes(builder);for(int i=0;i<count;++i)boxes.Add(i,mesh.Box(i));bvh_builders_on<T,N>(boxes,builder,mode,mesh);
 }
}
void bvh_builders_case(std::istream& input) {
 int dimension,precision;input>>dimension>>precision;
 if(precision==0){if(dimension==2)bvh_builders_run<double,2>(input);else if(dimension==3)bvh_builders_run<double,3>(input);else bvh_builders_run<double,4>(input);}
 else{if(dimension==2)bvh_builders_run<float,2>(input);else if(dimension==3)bvh_builders_run<float,3>(input);else bvh_builders_run<float,4>(input);}
}
// Point selector: metric = point-box square distance, closer first, pruned beyond MaxSq,
// whole branches accepted within AcceptSq; stops after StopCount Accept calls.
template<class T,int N> class ProbeLogSelector : public BVH_Traverse<T,N,void,T> {
public:
 typedef typename BVH_Traverse<T,N,void,T>::BVH_VecNt Vec;
 Vec Point;T MaxSq=T(0),AcceptSq=T(0);int StopCount=0,Count=0;mutable std::ostringstream Log;
 bool RejectNode(const Vec& mn,const Vec& mx,T& m) const override {m=BVH_Tools<T,N>::PointBoxSquareDistance(Point,mn,mx);Log<<"r "<<bits(double(m))<<' ';return m>MaxSq;}
 bool AcceptMetric(const T& m) const override {return m<=AcceptSq;}
 bool IsMetricBetter(const T& l,const T& r) const override {return l<r;}
 bool RejectMetric(const T& m) const override {return m>MaxSq;}
 bool Stop() const override {return StopCount>0&&Count>=StopCount;}
 bool Accept(const int i,const T& m) override {++Count;Log<<"a "<<i<<' '<<bits(double(m))<<' ';return i%3!=0;}
};
// Box selector with the default (left-to-right, never pruned, never stopped) rules.
template<class T,int N> class ProbeBoxSelector : public BVH_Traverse<T,N,void,T> {
public:
 typedef typename BVH_Traverse<T,N,void,T>::BVH_VecNt Vec;
 BVH_Box<T,N> Query;std::ostringstream Log;
 bool RejectNode(const Vec& mn,const Vec& mx,T& m) const override {m=T(0);return Query.IsOut(mn,mx);}
 bool Accept(const int i,const T&) override {Log<<i<<' ';return true;}
};
// Pair selector: metric = box-box square distance, closer first, pruned beyond PairSq.
template<class T,int N> class ProbePairSelector : public BVH_PairTraverse<T,N,void,T> {
public:
 typedef typename BVH_PairTraverse<T,N,void,T>::BVH_VecNt Vec;
 T PairSq=T(0);int StopCount=0,Count=0;mutable std::ostringstream Log;
 bool RejectNode(const Vec& a,const Vec& b,const Vec& c,const Vec& d,T& m) const override {m=BVH_Tools<T,N>::BoxBoxSquareDistance(a,b,c,d);Log<<"r "<<bits(double(m))<<' ';return m>PairSq;}
 bool IsMetricBetter(const T& l,const T& r) const override {return l<r;}
 bool RejectMetric(const T& m) const override {return m>PairSq;}
 bool Stop() const override {return StopCount>0&&Count>=StopCount;}
 bool Accept(const int i,const int j) override {++Count;Log<<"a "<<i<<' '<<j<<' ';return (i+j)%2==0;}
};
template<class T,int N> T bvh_query_triangle_distance(const typename BVH::VectorType<T,N>::Type& p,const BVH_Triangulation<T,N>& mesh,int index) {
 const BVH_Vec4i& e=mesh.Elements.Value(index);
 return BVH_Tools<T,N>::PointTriangleSquareDistance(p,mesh.Vertices.Value(e.x()),mesh.Vertices.Value(e.y()),mesh.Vertices.Value(e.z()));
}
template<class T,int N> class ProbePointDistance : public BVH_Distance<T,N,typename BVH::VectorType<T,N>::Type,BVH_Triangulation<T,N>> {
public:
 typedef typename BVH::VectorType<T,N>::Type Vec;std::ostringstream Log;
 bool RejectNode(const Vec& mn,const Vec& mx,T& m) const override {m=BVH_Tools<T,N>::PointBoxSquareDistance(this->myObject,mn,mx);return this->RejectMetric(m);}
 bool Accept(const int i,const T&) override {
  T d=bvh_query_triangle_distance<T,N>(this->myObject,*this->myBVHSet,i);Log<<i<<' ';
  if(d<this->myDistance){this->myDistance=d;return true;}return false;
 }
};
template<class T,int N> class ProbePairDistance : public BVH_PairDistance<T,N,BVH_Triangulation<T,N>> {
public:
 std::ostringstream Log;
 T Candidate(const BVH_Triangulation<T,N>& from,int i,const BVH_Triangulation<T,N>& to,int j) const {
  const BVH_Vec4i& e=from.Elements.Value(i);T best=std::numeric_limits<T>::max();
  for(int k=0;k<3;++k){T d=bvh_query_triangle_distance<T,N>(from.Vertices.Value(e[k]),to,j);if(d<best)best=d;}
  return best;
 }
 bool Accept(const int i,const int j) override {
  T d=Candidate(*this->myBVHSet1,i,*this->myBVHSet2,j);T other=Candidate(*this->myBVHSet2,j,*this->myBVHSet1,i);
  if(other<d)d=other;Log<<i<<' '<<j<<' ';
  if(d<this->myDistance){this->myDistance=d;return true;}return false;
 }
};
// input: kind leaf count1 count2 stop point(4) maxsq acceptsq pairsq, then both vertex lists.
template<class T,int N> void bvh_traverse_run(std::istream& input) {
 int kind,leaf,count1,count2,stop;double p[4],maxsq,acceptsq,pairsq;input>>kind>>leaf>>count1>>count2>>stop;
 for(double& value:p)input>>value;input>>maxsq>>acceptsq>>pairsq;
 BVH_Triangulation<T,N> a(bvh_query_builder<T,N>(kind,leaf,32,0)),b(bvh_query_builder<T,N>(kind,leaf,32,0));
 bvh_fill(a,input,count1);bvh_fill(b,input,count2);a.MarkDirty();b.MarkDirty();
 opencascade::handle<BVH_Tree<T,N>> ta=a.BVH(),tb=b.BVH();bvh_nodes_out(*ta,N);bvh_nodes_out(*tb,N);
 ProbeLogSelector<T,N> single;single.Point=bvh_vec<T,N>(p);single.MaxSq=T(maxsq);single.AcceptSq=T(acceptsq);single.StopCount=stop;
 std::cout<<single.Select(ta)<<' '<<single.Log.str();
 ProbeBoxSelector<T,N> boxes;boxes.Query=BVH_Box<T,N>(single.Point);boxes.Query.Add(single.Point+typename ProbeLogSelector<T,N>::Vec(T(acceptsq)));
 std::cout<<boxes.Select(ta)<<' '<<boxes.Log.str();
 ProbePairSelector<T,N> pairs;pairs.PairSq=T(pairsq);pairs.StopCount=stop;std::cout<<pairs.Select(ta,tb)<<' '<<pairs.Log.str();
 ProbePairSelector<T,N> self;self.PairSq=T(pairsq);std::cout<<self.Select(ta,ta)<<' '<<self.Log.str();
 ProbePointDistance<T,N> distance;distance.SetObject(single.Point);distance.SetBVHSet(&a);bvh_bits(distance.ComputeDistance());
 std::cout<<distance.IsDone()<<' '<<distance.Log.str();
 ProbePointDistance<T,N> unset;bvh_bits(unset.ComputeDistance());std::cout<<unset.IsDone()<<' ';
 ProbePairDistance<T,N> pair;pair.SetBVHSets(&a,&b);bvh_bits(pair.ComputeDistance());std::cout<<pair.IsDone()<<' '<<pair.Log.str()<<'\n';
}
void bvh_traverse_case(std::istream& input) {
 int dimension,precision;input>>dimension>>precision;
 if(precision==0){if(dimension==2)bvh_traverse_run<double,2>(input);else if(dimension==3)bvh_traverse_run<double,3>(input);else bvh_traverse_run<double,4>(input);}
 else{if(dimension==2)bvh_traverse_run<float,2>(input);else if(dimension==3)bvh_traverse_run<float,3>(input);else bvh_traverse_run<float,4>(input);}
}
// v: point, a, b, c, box minimum, box maximum (four values each).
template<class T,int N> void bvh_nearest_run(const double* v) {
 auto p=bvh_vec<T,N>(v),a=bvh_vec<T,N>(v+4),b=bvh_vec<T,N>(v+8),c=bvh_vec<T,N>(v+12);
 bvh_vec_out(BVH::DirectionToNearestPoint<T,N>(p,a,b,c),N);bvh_bits(BVH::DistanceToBox<T,N>(p,bvh_vec<T,N>(v+16),bvh_vec<T,N>(v+20)));std::cout<<'\n';
}
void bvh_nearest_case(std::istream& input) {
 int dimension,precision;input>>dimension>>precision;double v[24];for(auto& value:v)input>>value;
 if(precision==0){if(dimension==3)bvh_nearest_run<double,3>(v);else bvh_nearest_run<double,4>(v);}
 else{if(dimension==3)bvh_nearest_run<float,3>(v);else bvh_nearest_run<float,4>(v);}
}
// A geometric object that is not a triangulation: SquareDistanceToObject skips it.
template<class T,int N> class ProbeBoxObject : public BVH_Object<T,N> {
public:
 BVH_Box<T,N> Bounds;
 BVH_Box<T,N> Box() const override {return Bounds;}
};
// The object tag: element w of a triangulation, -1 for other objects or empty meshes.
template<class T,int N> int bvh_object_tag(const opencascade::handle<BVH_Object<T,N>>& object) {
 const BVH_Triangulation<T,N>* mesh=dynamic_cast<const BVH_Triangulation<T,N>*>(object.get());
 return mesh!=nullptr&&mesh->Elements.Size()>0?mesh->Elements.Value(0).w():-1;
}
// input: maxsize sign parallel geomdirty meshdirty prebuild extra builder objects counts..., vertices.
template<class T,int N> void bvh_field_run(std::istream& input) {
 int maxsize,sign,parallel,geomdirty,meshdirty,prebuild,extra,kind,objects;
 input>>maxsize>>sign>>parallel>>geomdirty>>meshdirty>>prebuild>>extra>>kind>>objects;
 std::vector<int> counts(objects);for(int& count:counts)input>>count;
 BVH_Geometry<T,N> geometry;if(kind!=0)geometry.SetBuilder(bvh_query_builder<T,N>(kind==1?0:2,1,32,0));
 for(int k=0;k<objects;++k){
  opencascade::handle<BVH_Triangulation<T,N>> mesh=new BVH_Triangulation<T,N>();bvh_fill(*mesh,input,counts[k]);
  for(int i=0;i<mesh->Elements.Size();++i)mesh->Elements.ChangeValue(i).w()=k;
  if(meshdirty!=0)mesh->MarkDirty();geometry.Objects().Append(mesh);
 }
 if(extra!=0){opencascade::handle<ProbeBoxObject<T,N>> box=new ProbeBoxObject<T,N>();double c[4]={0.5,0.5,0.5,0.5};
  box->Bounds=BVH_Box<T,N>(bvh_vec<T,N>(c));geometry.Objects().Append(box);}
 if(geomdirty!=0)geometry.MarkDirty();
 if(prebuild!=0){geometry.BVH();for(int k=0;k<geometry.Size();++k){auto* mesh=dynamic_cast<BVH_Triangulation<T,N>*>(geometry.Objects()(k).get());if(mesh!=nullptr)mesh->BVH();}}
 BVH_DistanceField<T,N> field(maxsize,sign!=0);field.SetParallel(parallel!=0);std::cout<<field.Build(geometry)<<' ';
 std::cout<<field.DimensionX()<<' '<<field.DimensionY()<<' '<<field.DimensionZ()<<' ';
 bvh_vec_out(field.CornerMin(),N);bvh_vec_out(field.CornerMax(),N);bvh_vec_out(field.VoxelSize(),N);
 for(int z=0;z<field.DimensionZ();++z)for(int y=0;y<field.DimensionY();++y)for(int x=0;x<field.DimensionX();++x)bvh_bits(field.Voxel(x,y,z));
 std::cout<<geometry.IsDirty()<<' ';
 for(int k=0;k<geometry.Size();++k)std::cout<<bvh_object_tag<T,N>(geometry.Objects()(k))<<' '<<geometry.Objects()(k)->IsDirty()<<' ';
 bvh_nodes_out(*geometry.BVH(),N);std::cout<<'\n';
}
void bvh_field_case(std::istream& input) {
 int dimension,precision;input>>dimension>>precision;
 if(precision==0){if(dimension==3)bvh_field_run<double,3>(input);else bvh_field_run<double,4>(input);}
 else{if(dimension==3)bvh_field_run<float,3>(input);else bvh_field_run<float,4>(input);}
}
