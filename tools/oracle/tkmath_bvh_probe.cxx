// BVH core oracle at pinned OCCT 3d097a0: boxes, rays, tools, sorters, binned builds,
// SAH, QBVH collapse, JSON dumps and 4D transforms, for float/double and N = 2..4.
#include <BVH_Box.hxx>
#include <BVH_Ray.hxx>
#include <BVH_Tools.hxx>
#include <BVH_Triangulation.hxx>
#include <BVH_QuickSorter.hxx>
#include <BVH_RadixSorter.hxx>
#include <BVH_BinnedBuilder.hxx>
#include <BVH_QuadTree.hxx>
#include <BVH_Properties.hxx>
#include <vector>
template<class T> void bvh_bits(T value) {std::cout<<bits(double(value))<<' ';}
template<class V> void bvh_vec_out(const V& value,int count) {for(int i=0;i<count;++i)bvh_bits(value[i]);}
template<class T,int N> typename BVH::VectorType<T,N>::Type bvh_vec(const double* v) {
 typename BVH::VectorType<T,N>::Type result;for(int i=0;i<N;++i)result[i]=T(v[i]);return result;
}
template<class T,int N> void bvh_box_out(const BVH_Box<T,N>& box) {bvh_vec_out(box.CornerMin(),N);bvh_vec_out(box.CornerMax(),N);}
template<class T,int N> void bvh_box_queries(const BVH_Box<T,N>& a,const BVH_Box<T,N>& b,const typename BVH::VectorType<T,N>::Type& p) {
 std::cout<<a.IsValid()<<' ';bvh_bits(a.Area());bvh_vec_out(a.Size(),N);bvh_vec_out(a.Center(),N);
 for(int axis=0;axis<N;++axis)bvh_bits(a.Center(axis));
 bool overlap=true;std::cout<<a.IsOut(b)<<' '<<a.IsOut(b.CornerMin(),b.CornerMax())<<' ';
 std::cout<<a.Contains(b,overlap)<<' '<<overlap<<' ';overlap=true;std::cout<<a.Contains(b.CornerMin(),b.CornerMax(),overlap)<<' '<<overlap<<' '<<a.IsOut(p)<<' ';
 BVH_Box<T,N> c=a;c.Combine(b);bvh_box_out(c);c=a;c.Add(p);bvh_box_out(c);
}
template<class T,int N> void bvh_tool_queries(const BVH_Box<T,N>& a,const BVH_Box<T,N>& b,const double* v) {
 typedef BVH_Tools<T,N> Tools;auto p=bvh_vec<T,N>(v+17),d=bvh_vec<T,N>(v+21);
 bvh_bits(Tools::PointBoxSquareDistance(p,a));bvh_bits(Tools::BoxBoxSquareDistance(a,b));bvh_vec_out(Tools::PointBoxProjection(p,a),N);
 BVH_Ray<T,N> ray(p,d);bvh_vec_out(ray.InvDirect,N);T enter=T(91),leave=T(92);
 std::cout<<Tools::RayBoxIntersection(ray,a,enter,leave)<<' ';bvh_bits(enter);bvh_bits(leave);enter=T(91);leave=T(92);
 std::cout<<Tools::RayBoxIntersection(p,d,a,enter,leave)<<' ';bvh_bits(enter);bvh_bits(leave);
 typename Tools::BVH_PrjStateInTriangle state=Tools::BVH_PrjStateInTriangle_EDGE;int first=-7,last=-7;
 auto t0=bvh_vec<T,N>(v+25),t1=bvh_vec<T,N>(v+29),t2=bvh_vec<T,N>(v+33);
 bvh_vec_out(Tools::PointTriangleProjection(p,t0,t1,t2,&state,&first,&last),N);
 std::cout<<int(state)<<' '<<first<<' '<<last<<' ';bvh_bits(Tools::PointTriangleSquareDistance(p,t0,t1,t2));
}
template<class T,int N> void bvh_json_queries(const BVH_Box<T,N>& a,int cut) {
 std::ostringstream dump;a.DumpJson(dump);std::string text=dump.str();
 BVH_Box<T,N> parsed(bvh_vec<T,N>(std::vector<double>(4,7.0).data()));Standard_SStream stream;
 stream<<(cut<0||cut>=int(text.size())?text:text.substr(0,cut));int position=1;
 try{bool ok=parsed.InitFromJson(stream,position);std::cout<<ok<<' '<<position<<' ';}catch(const Standard_Failure& failure){std::cout<<failure.ExceptionType()<<' ';}
 bvh_box_out(parsed);std::cout<<text;
}
// v: flag, Amin, Amax, Bmin, Bmax, P, D, T0, T1, T2 (four values each), matrix (16), cut.
template<class T,int N> void bvh_box_run(const double* v) {
 BVH_Box<T,N> a=v[0]!=0?BVH_Box<T,N>():BVH_Box<T,N>(bvh_vec<T,N>(v+1),bvh_vec<T,N>(v+5));
 BVH_Box<T,N> b(bvh_vec<T,N>(v+9),bvh_vec<T,N>(v+13));bvh_box_queries(a,b,bvh_vec<T,N>(v+17));bvh_tool_queries(a,b,v);
 if constexpr(N==3){NCollection_Mat4<T> m;for(int i=0;i<16;++i)m.ChangeData()[i]=T(v[37+i]);bvh_box_out(a.Transformed(m));BVH_Box<T,N> c=a;c.Transform(m);bvh_box_out(c);}
 bvh_json_queries(a,int(v[53]));std::cout<<'\n';
}
void bvh_box_case(std::istream& input) {
 int dimension,precision;input>>dimension>>precision;double v[54];for(auto& value:v)input>>value;
 if(precision==0){if(dimension==2)bvh_box_run<double,2>(v);else if(dimension==3)bvh_box_run<double,3>(v);else bvh_box_run<double,4>(v);}
 else{if(dimension==2)bvh_box_run<float,2>(v);else if(dimension==3)bvh_box_run<float,3>(v);else bvh_box_run<float,4>(v);}
}
void bvh_int_floor_case(std::istream& input) {
 double a,b;input>>a>>b;double x=a/b;float f=float(a)/float(b);
 std::cout<<BVH::IntFloor(x)<<' '<<BVH::IntFloor(f)<<'\n';
}
template<class T,int N> void bvh_fill(BVH_Triangulation<T,N>& set,std::istream& input,int count) {
 for(int i=0;i<3*count;++i){double c[4]={0,0,0,0};for(int k=0;k<N;++k)input>>c[k];BVH::Array<T,N>::Append(set.Vertices,bvh_vec<T,N>(c));}
 for(int i=0;i<count;++i)BVH::Array<int,4>::Append(set.Elements,BVH_Vec4i(3*i,3*i+1,3*i+2,0));
}
template<class T,int N> void bvh_order_out(const BVH_Triangulation<T,N>& set) {for(int i=0;i<set.Size();++i)std::cout<<set.Elements.Value(i).x()/3<<' ';}
// input: axis start last boxmode parallel count, then 3*count vertices of N coordinates.
template<class T,int N> void bvh_sort_run(std::istream& input) {
 int axis,start,last,mode,parallel,count;input>>axis>>start>>last>>mode>>parallel>>count;
 BVH_Triangulation<T,N> quick;bvh_fill(quick,input,count);BVH_Triangulation<T,N> radix;radix.Vertices=quick.Vertices;radix.Elements=quick.Elements;
 {BVH_QuickSorter<T,N> sorter(axis);if(last<0)sorter.Perform(&quick);else sorter.Perform(&quick,start,last);bvh_order_out(quick);}
 BVH_Box<T,N> box;if(mode==0){radix.MarkDirty();box=radix.Box();}bvh_box_out(box);
 BVH_RadixSorter<T,N> sorter(box);sorter.SetParallel(parallel!=0);if(last<0)sorter.Perform(&radix);else sorter.Perform(&radix,start,last);
 for(const auto& link:sorter.EncodedLinks())std::cout<<link.first<<' '<<link.second<<' ';bvh_order_out(radix);std::cout<<'\n';
}
void bvh_sort_case(std::istream& input) {
 int dimension,precision;input>>dimension>>precision;
 if(precision==0){if(dimension==2)bvh_sort_run<double,2>(input);else if(dimension==3)bvh_sort_run<double,3>(input);else bvh_sort_run<double,4>(input);}
 else{if(dimension==2)bvh_sort_run<float,2>(input);else if(dimension==3)bvh_sort_run<float,3>(input);else bvh_sort_run<float,4>(input);}
}
template<class Tree> void bvh_nodes_out(const Tree& tree,int dimension) {
 std::cout<<tree.Length()<<' '<<tree.Depth()<<' ';
 for(int i=0;i<tree.Length();++i){const auto& r=tree.NodeInfoBuffer().Value(i);std::cout<<r.x()<<' '<<r.y()<<' '<<r.z()<<' '<<r.w()<<' ';bvh_vec_out(tree.MinPoint(i),dimension);bvh_vec_out(tree.MaxPoint(i),dimension);}
}
template<class T,int N,int Bins> opencascade::handle<BVH_Builder<T,N>> bvh_binned(int leaf,int depth,bool main) {return new BVH_BinnedBuilder<T,N,Bins>(leaf,depth,main);}
template<class T,int N> opencascade::handle<BVH_Builder<T,N>> bvh_builder(int bins,int leaf,int depth,bool main) {
 switch(bins){case 2:return bvh_binned<T,N,2>(leaf,depth,main);case 4:return bvh_binned<T,N,4>(leaf,depth,main);case 16:return bvh_binned<T,N,16>(leaf,depth,main);
 case 32:return bvh_binned<T,N,32>(leaf,depth,main);default:return bvh_binned<T,N,48>(leaf,depth,main);}
}
// input: bins(0 = default primitive set) leaf depth mainaxis count, then 3*count vertices.
template<class T,int N> void bvh_build_run(std::istream& input) {
 int bins,leaf,depth,main,count;input>>bins>>leaf>>depth>>main>>count;
 BVH_Triangulation<T,N> set;if(bins!=0)set.SetBuilder(bvh_builder<T,N>(bins,leaf,depth,main!=0));bvh_fill(set,input,count);
 bvh_box_out(set.Box());set.MarkDirty();const auto& tree=set.BVH();bvh_nodes_out(*tree,N);bvh_order_out(set);
 if(tree->Length()>0)bvh_bits(tree->EstimateSAH());opencascade::handle<BVH_Tree<T,N,BVH_QuadTree>> quad=tree->CollapseToQuadTree();
 bvh_nodes_out(*quad,N);if(quad->Length()>0)std::cout<<quad->template Child<3>(0)<<' ';bvh_box_out(set.Box());
 std::ostringstream dump;tree->DumpJson(dump);std::cout<<dump.str()<<'\n';
}
void bvh_build_case(std::istream& input) {
 int dimension,precision;input>>dimension>>precision;
 if(precision==0){if(dimension==2)bvh_build_run<double,2>(input);else if(dimension==3)bvh_build_run<double,3>(input);else bvh_build_run<double,4>(input);}
 else{if(dimension==2)bvh_build_run<float,2>(input);else if(dimension==3)bvh_build_run<float,3>(input);else bvh_build_run<float,4>(input);}
}
// v: matrix (16, column-major), box minimum (4), box maximum (4).
template<class T> void bvh_transform_run(const double* v) {
 NCollection_Mat4<T> m;for(int i=0;i<16;++i)m.ChangeData()[i]=T(v[i]);
 NCollection_Mat4<T> inverse;T determinant=T(5);std::cout<<m.Inverted(inverse,determinant)<<' ';bvh_bits(determinant);bvh_vec_out(inverse.GetData(),16);
 BVH_Transform<T,4> constructed(m);bvh_vec_out(constructed.Inversed().GetData(),16);
 BVH_Transform<T,4> assigned;assigned.SetTransform(m);bvh_vec_out(assigned.Inversed().GetData(),16);
 BVH_Box<T,4> box(bvh_vec<T,4>(v+16),bvh_vec<T,4>(v+20));bvh_box_out(assigned.Apply(box));std::cout<<'\n';
}
void bvh_transform_case(std::istream& input) {
 int precision;input>>precision;double v[24];for(auto& value:v)input>>value;
 if(precision==0)bvh_transform_run<double>(v);else bvh_transform_run<float>(v);
}
