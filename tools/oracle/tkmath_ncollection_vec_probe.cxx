// NCollection_Vec2/Vec3/Vec4, NCollection_Mat3 and NCollection_Mat4 oracle at pinned 3d097a0, for
// float, double and int elements. Floats print as the bits of their exact double promotion.
#include <NCollection_Mat4.hxx>
template<class T> void ncv_out(T value){if constexpr(std::is_same_v<T,int>)std::cout<<value<<' ';else std::cout<<bits(double(value))<<' ';}
template<class V> void ncv_vec(const V& v){for(int i=0;i<V::Length();++i)ncv_out(v[i]);}
template<class T> void ncv_json(const T& value){std::ostringstream dump;value.DumpJson(dump,-1);std::cout<<dump.str()<<" | ";}
template<class V,class T> void ncv_common(const V& a,const V& b,T s){
 ncv_vec(a+b);ncv_vec(a-b);ncv_vec(-a);ncv_vec(a*b);ncv_vec(a*s);ncv_vec(a.Multiplied(s));ncv_vec(a/b);ncv_vec(a/s);
 V c=a;c+=b;ncv_vec(c);c=a;c-=b;ncv_vec(c);c=a;c*=b;ncv_vec(c);c=a;c*=s;ncv_vec(c);c=a;c.Multiply(s);ncv_vec(c);c=a;c/=s;ncv_vec(c);c=a;c/=b;ncv_vec(c);
 ncv_vec(a.cwiseMin(b));ncv_vec(a.cwiseMax(b));ncv_vec(a.cwiseAbs());ncv_out(a.maxComp());ncv_out(a.minComp());ncv_out(a.Dot(b));
 std::cout<<a.IsEqual(b)<<' '<<(a==b)<<' '<<(a!=b)<<' '<<a.IsEqual(a)<<' ';
}
template<class T> void ncv_vec2(const T* p,const T* q,T s){
 NCollection_Vec2<T> a(p[0],p[1]),b(q[0],q[1]);ncv_common(a,b,s);ncv_out(a.Modulus());ncv_out(a.SquareModulus());
 ncv_vec(a.xy());ncv_vec(a.yx());ncv_vec(NCollection_Vec2<T>(s));ncv_vec(NCollection_Vec2<T>::DX());ncv_vec(NCollection_Vec2<T>::DY());
 NCollection_Vec2<T> c;c.SetValues(q[1],q[0]);ncv_vec(c);ncv_vec(NCollection_Vec2<float>(a));ncv_vec(NCollection_Vec2<double>(a));ncv_vec(NCollection_Vec2<int>(a));
 if constexpr(!std::is_same_v<T,int>)ncv_json(a);
}
template<class T> void ncv_vec3(const T* p,const T* q,T s,T t){
 NCollection_Vec3<T> a(p[0],p[1],p[2]),b(q[0],q[1],q[2]);ncv_common(a,b,s);ncv_out(a.Modulus());ncv_out(a.SquareModulus());
 ncv_vec(a.Normalized());NCollection_Vec3<T> c=a;c.Normalize();ncv_vec(c);ncv_vec(NCollection_Vec3<T>::Cross(a,b));ncv_vec(NCollection_Vec3<T>::GetLERP(a,b,t));
 ncv_vec(a.xy());ncv_vec(a.yx());ncv_vec(a.xz());ncv_vec(a.zx());ncv_vec(a.yz());ncv_vec(a.zy());
 ncv_vec(a.xyz());ncv_vec(a.xzy());ncv_vec(a.yxz());ncv_vec(a.yzx());ncv_vec(a.zyx());ncv_vec(a.zxy());ncv_out(a.r());ncv_out(a.g());ncv_out(a.b());
 ncv_vec(NCollection_Vec3<T>(s));ncv_vec(NCollection_Vec3<T>(NCollection_Vec2<T>(q[0],q[1])));ncv_vec(NCollection_Vec3<T>(NCollection_Vec2<T>(q[0],q[1]),s));
 ncv_vec(NCollection_Vec3<T>::DX());ncv_vec(NCollection_Vec3<T>::DY());ncv_vec(NCollection_Vec3<T>::DZ());
 c.SetValues(q[2],q[1],q[0]);ncv_vec(c);c.SetValues(NCollection_Vec2<T>(p[1],p[0]),s);ncv_vec(c);
 ncv_vec(NCollection_Vec3<float>(a));ncv_vec(NCollection_Vec3<double>(a));ncv_vec(NCollection_Vec3<int>(a));
 if constexpr(!std::is_same_v<T,int>)ncv_json(a);
}
template<class T> void ncv_vec4(const T* p,const T* q,T s){
 NCollection_Vec4<T> a(p[0],p[1],p[2],p[3]),b(q[0],q[1],q[2],q[3]);ncv_common(a,b,s);
 ncv_vec(a.xy());ncv_vec(a.yx());ncv_vec(a.xz());ncv_vec(a.zx());ncv_vec(a.xw());ncv_vec(a.wx());ncv_vec(a.yz());ncv_vec(a.zy());ncv_vec(a.yw());ncv_vec(a.wy());ncv_vec(a.zw());ncv_vec(a.wz());
 ncv_vec(a.xyz());ncv_vec(a.xzy());ncv_vec(a.yxz());ncv_vec(a.yzx());ncv_vec(a.zyx());ncv_vec(a.zxy());
 ncv_vec(a.xyw());ncv_vec(a.xwy());ncv_vec(a.yxw());ncv_vec(a.ywx());ncv_vec(a.wyx());ncv_vec(a.wxy());
 ncv_vec(a.xzw());ncv_vec(a.xwz());ncv_vec(a.zxw());ncv_vec(a.zwx());ncv_vec(a.wzx());ncv_vec(a.wxz());
 ncv_vec(a.yzw());ncv_vec(a.ywz());ncv_vec(a.zyw());ncv_vec(a.zwy());ncv_vec(a.wzy());ncv_vec(a.wyz());
 ncv_vec(a.rgb());ncv_vec(a.rbg());ncv_vec(a.grb());ncv_vec(a.gbr());ncv_vec(a.bgr());ncv_vec(a.brg());ncv_out(a.r());ncv_out(a.g());ncv_out(a.b());ncv_out(a.a());
 ncv_vec(NCollection_Vec4<T>(s));ncv_vec(NCollection_Vec4<T>(NCollection_Vec2<T>(q[0],q[1])));ncv_vec(NCollection_Vec4<T>(NCollection_Vec3<T>(q[0],q[1],q[2])));
 ncv_vec(NCollection_Vec4<T>(NCollection_Vec3<T>(q[0],q[1],q[2]),s));NCollection_Vec4<T> c;c.SetValues(q[3],q[2],q[1],q[0]);ncv_vec(c);
 c.SetValues(NCollection_Vec3<T>(p[2],p[1],p[0]),s);ncv_vec(c);ncv_vec(NCollection_Vec4<float>(a));ncv_vec(NCollection_Vec4<double>(a));ncv_vec(NCollection_Vec4<int>(a));
 if constexpr(!std::is_same_v<T,int>)ncv_json(a);
}
template<class T> void ncv_typed(std::istream& in){
 double raw[10];for(double& v:raw)in>>v;T p[4],q[4];for(int i=0;i<4;++i){p[i]=T(raw[i]);q[i]=T(raw[4+i]);}
 ncv_vec2<T>(p,q,T(raw[8]));ncv_vec3<T>(p,q,T(raw[8]),T(raw[9]));ncv_vec4<T>(p,q,T(raw[8]));std::cout<<'\n';
}
void ncollection_vec_case(std::istream& in){
 double kind;in>>kind;if(kind==0)ncv_typed<float>(in);else if(kind==1)ncv_typed<double>(in);else ncv_typed<int>(in);
}
// NCollection_Mat3 has no Rows()/Cols(); the dimension is the vector length of a row.
template<class M> void ncv_mat(const M& m){const size_t n=size_t(decltype(m.GetRow(0))::Length());for(size_t r=0;r<n;++r)for(size_t c=0;c<n;++c)ncv_out(m.GetValue(r,c));}
template<class M,class V,class T> void ncv_matrix_common(const M& a,const M& b,const V& v,T s){
 ncv_mat(a*b);ncv_mat(M::Multiply(a,b));ncv_mat(a.Multiplied(b));M c=a;c.Multiply(b);ncv_mat(c);c=a;c*=b;ncv_mat(c);ncv_vec(a*v);
 ncv_mat(a*s);ncv_mat(a.Multiplied(s));c=a;c.Multiply(s);ncv_mat(c);c=a;c*=s;ncv_mat(c);ncv_mat(a/s);ncv_mat(a.Divided(s));c=a;c.Divide(s);ncv_mat(c);c=a;c/=s;ncv_mat(c);
 ncv_mat(a+b);ncv_mat(a.Added(b));ncv_mat(a-b);ncv_mat(a.Subtracted(b));c=a;c+=b;ncv_mat(c);c=a;c-=b;ncv_mat(c);ncv_mat(-a);ncv_mat(a.Negated());
 ncv_mat(a.Transposed());c=a;c.Transpose();ncv_mat(c);ncv_mat(a.Adjoint());
 M inv=b;T det=T(91);std::cout<<a.Inverted(inv,det)<<' ';ncv_out(det);ncv_mat(inv);inv=b;std::cout<<a.Inverted(inv)<<' ';ncv_mat(inv);
 try{ncv_mat(a.Inverted());}catch(const Standard_Failure& f){std::cout<<f.ExceptionType()<<' ';}
 std::cout<<a.IsZero()<<' '<<a.IsIdentity()<<' '<<a.IsEqual(b)<<' '<<(a==a)<<' '<<(a!=b)<<' '<<M::Zero().IsZero()<<' '<<M::Identity().IsIdentity()<<' ';
 ncv_vec(a.GetRow(1));ncv_vec(a.GetColumn(2));ncv_vec(a.GetDiagonal());c=a;c.SetRow(0,v);c.SetColumn(1,v);c.SetDiagonal(v);ncv_mat(c);
 c=a;c.InitZero();ncv_mat(c);c.InitIdentity();ncv_mat(c);ncv_json(a);
}
template<class T> void ncv_matrices(std::istream& in){
 double raw[37];for(double& v:raw)in>>v;NCollection_Mat4<T> a,b;
 for(int i=0;i<16;++i){a.ChangeData()[i]=T(raw[i]);b.ChangeData()[i]=T(raw[16+i]);}
 NCollection_Vec4<T> v{T(raw[32]),T(raw[33]),T(raw[34]),T(raw[35])};const T s=T(raw[36]);
 ncv_matrix_common(a,b,v,s);ncv_out(a.DeterminantMat3());ncv_mat(a.GetMat3());NCollection_Mat4<T> c=a;c.Translate(v.xyz());ncv_mat(c);
 c=a;c.SetRow(2,v.xyz());c.SetColumn(3,v.yzx());c.SetDiagonal(v.zxy());ncv_mat(c);ncv_mat(NCollection_Mat4<float>(a));ncv_mat(NCollection_Mat4<double>(a));
 c.ConvertFrom(NCollection_Mat4<float>(b));ncv_mat(c);
 const NCollection_Mat3<T> a3=a.GetMat3(),b3=b.GetMat3();ncv_matrix_common(a3,b3,v.xyz(),s);ncv_out(a3.Determinant());
 std::cout<<'\n';
}
void ncollection_mat_case(std::istream& in){double kind;in>>kind;if(kind==0)ncv_matrices<float>(in);else ncv_matrices<double>(in);}
