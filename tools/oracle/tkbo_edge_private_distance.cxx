void ee_distance_snapshot(double distance,double parameter,double best,double first,double second){for(double v:{distance,parameter,best,first,second})bo_real(v);}
void ee_distance(const std::vector<double>& v){
 auto source=ee_geometry_fixture(int(v[0]),.5),target=ee_geometry_fixture(int(v[1]),.5);GeomAPI_ProjectPointOnCurve projection;projection.Init(target,-2,3);
 bool maximum=v[6]!=0;int sign=maximum?1:-1;double distance=17,parameter=19,best=maximum?-1:1e100,first=23,second=29;
 bo_int(DistPC((v[2]+v[3])*.5,source,v[4],projection,distance,parameter,sign));ee_distance_snapshot(distance,parameter,best,first,second);
 bo_int(DistPC(v[2],source,v[4],projection,distance,parameter,best,first,second,sign));ee_distance_snapshot(distance,parameter,best,first,second);
 bo_int(FindDistPC(v[2],v[3],source,v[4],v[5],projection,best,first,second,maximum));ee_distance_snapshot(distance,parameter,best,first,second);
 bo_int(projection.NbPoints());if(projection.NbPoints()){bo_real(projection.LowerDistance());bo_real(projection.LowerDistanceParameter());}
}
