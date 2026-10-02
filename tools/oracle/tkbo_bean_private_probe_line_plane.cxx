void bean_line_plane(const std::vector<double>& v){
 gp_Lin line(gp_Pnt(v[0],v[1],v[2]),gp_Dir(v[3],v[4],v[5]));gp_Pln plane;plane.SetLocation(gp_Pnt(0.,0.,v[6]));if(v[7]!=0.)plane.UReverse();
 TopoDS_Edge edge=BRepBuilderAPI_MakeEdge(line,-10.,10.);TopoDS_Face face=BRepBuilderAPI_MakeFace(plane);
 BRepAdaptor_Curve curve(edge);BRepAdaptor_Surface surface(face,false);
 IntTools_BeanFaceIntersector intersector(curve,surface,v[8],v[9],v[10],v[11],v[12],v[13],v[14],v[15]);
 intersector.Perform();bo_int(intersector.IsDone());bo_real(intersector.MinimalSquareDistance());bo_int(intersector.Result().Length());for(const auto& range:intersector.Result())bo_range(range);
}
