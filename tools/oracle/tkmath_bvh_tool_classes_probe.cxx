// BVH namespace tool classes at pinned OCCT 3d097a0: ToBndBox (all four overloads),
// VecComp::Get, UnitVector::DX/DY/DZ, MatrixOp<T,4>::Inverse/Multiply,
// SurfaceCalculator::Area, BVH_AxisSelector::MainAxis, CenterAxis::Center and
// BoxMinMax::CwiseMin/CwiseMax, for float and double and N = 2..4.
#include <BVH_Box.hxx>
#include <BVH_BinnedBuilder.hxx>
#include <BVH_Properties.hxx>
void bvh_tool_bnd_out(const Bnd_Box& box) {
 double c[6];box.Get(c[0],c[1],c[2],c[3],c[4],c[5]);for(double value:c)std::cout<<bits(value)<<' ';
}
template<class T,int N> typename BVH::VectorType<T,N>::Type bvh_tool_vec(const double* v) {
 typename BVH::VectorType<T,N>::Type result;for(int i=0;i<N;++i)result[i]=T(v[i]);return result;
}
template<class T,int N> void bvh_tool_vec_out(const typename BVH::VectorType<T,N>::Type& value) {
 for(int i=0;i<N;++i)std::cout<<bits(double(value[i]))<<' ';
}
// Per dimension: ToBndBox, VecComp for axes -1..4, unit vectors, SAH area, main axis,
// box centers per axis and the in-place component-wise min/max.
template<class T,int N> void bvh_tool_dimension(const double* a,const double* b) {
 auto p=bvh_tool_vec<T,N>(a),q=bvh_tool_vec<T,N>(b);bvh_tool_bnd_out(BVH::ToBndBox(p,q));
 for(int axis=-1;axis<=4;++axis)std::cout<<bits(double(BVH::VecComp<T,N>::Get(p,axis)))<<' ';
 bvh_tool_vec_out<T,N>(BVH::UnitVector<T,N>::DX());bvh_tool_vec_out<T,N>(BVH::UnitVector<T,N>::DY());
 bvh_tool_vec_out<T,N>(BVH::UnitVector<T,N>::DZ());std::cout<<bits(double(BVH::SurfaceCalculator<T,N>::Area(p)))<<' ';
 std::cout<<BVH::BVH_AxisSelector<T,N>::MainAxis(p)<<' ';BVH_Box<T,N> box(p,q);
 for(int axis=0;axis<N;++axis)std::cout<<bits(double(BVH::CenterAxis<T,N>::Center(box,axis)))<<' ';
 auto low=p,high=p;BVH::BoxMinMax<T,N>::CwiseMin(low,q);BVH::BoxMinMax<T,N>::CwiseMax(high,q);
 bvh_tool_vec_out<T,N>(low);bvh_tool_vec_out<T,N>(high);
}
// v: a (4), b (4), matrix (16, column-major).
template<class T> void bvh_tool_run(const double* v) {
 bvh_tool_bnd_out(BVH::ToBndBox(T(v[0]),T(v[4])));
 bvh_tool_dimension<T,2>(v,v+4);bvh_tool_dimension<T,3>(v,v+4);bvh_tool_dimension<T,4>(v,v+4);
 NCollection_Mat4<T> m;for(int i=0;i<16;++i)m.ChangeData()[i]=T(v[8+i]);NCollection_Mat4<T> inverse;
 BVH::MatrixOp<T,4>::Inverse(m,inverse);for(int i=0;i<16;++i)std::cout<<bits(double(inverse.GetData()[i]))<<' ';
 bvh_tool_vec_out<T,4>(BVH::MatrixOp<T,4>::Multiply(m,bvh_tool_vec<T,4>(v)));std::cout<<'\n';
}
void bvh_tool_classes_case(std::istream& input) {
 int precision;input>>precision;double v[24];for(auto& value:v)input>>value;
 if(precision==0)bvh_tool_run<double>(v);else bvh_tool_run<float>(v);
}
