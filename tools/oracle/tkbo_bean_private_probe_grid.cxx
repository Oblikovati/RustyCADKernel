Handle(Geom_BSplineSurface) bean_grid_spline(int degree,int spans,double height){
 int count=degree+spans;NCollection_Array2<gp_Pnt> poles(1,count,1,count);
 for(int i=1;i<=count;++i)for(int j=1;j<=count;++j)poles(i,j)=gp_Pnt(i,j,(i+j)%2==0?height:-height);
 NCollection_Array1<double> knots(1,spans+1);NCollection_Array1<int> mult(1,spans+1);
 for(int i=0;i<=spans;++i){knots(i+1)=i;mult(i+1)=(i==0||i==spans)?degree+1:1;}
 return new Geom_BSplineSurface(poles,knots,knots,mult,mult,degree,degree,false,false);
}
void bean_grid_snapshot(const IntTools_SurfaceRangeLocalizeData& cache){
 int nu=cache.GetRangeUGrid(),nv=cache.GetRangeVGrid();bo_int(nu);bo_int(nv);bo_real(cache.GetGridDeflection());
 for(int i=1;i<=nu;++i)bo_real(cache.GetUParam(i));for(int j=1;j<=nv;++j)bo_real(cache.GetVParam(j));
 for(int i=1;i<=nu;++i)for(int j=1;j<=nv;++j){auto p=cache.GetGridPoint(i,j);bo_real(p.X());bo_real(p.Y());bo_real(p.Z());}
}
void bean_grid(const std::vector<double>& v){
 auto spline=bean_grid_spline(int(v[0]),int(v[1]),v[2]);TopoDS_Face face;BRep_Builder().MakeFace(face,spline,1e-7);BRepAdaptor_Surface surface(face,false);IntTools_SurfaceRangeLocalizeData cache(3,3,0.01,0.01);
 ComputeGridPoints(surface,spline,v[3],v[4],v[5],v[6],v[7],cache);bean_grid_snapshot(cache);
 ComputeGridPoints(surface,spline,0.,v[1],0.,v[1],v[7],cache);bean_grid_snapshot(cache);
}
