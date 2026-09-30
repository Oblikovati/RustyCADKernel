#include <BRepMesh_DefaultRangeSplitter.hxx>
#include <BRepMesh_ConeRangeSplitter.hxx>
#include <Geom_ConicalSurface.hxx>
#include <Geom_CylindricalSurface.hxx>
#include <Geom_BezierSurface.hxx>
#include <NCollection_Array2.hxx>
static IMeshData::IFaceHandle range_face(int kind,double tolerance) {
  occ::handle<Geom_Surface> surface;
  if(kind==0)surface=new Geom_ConicalSurface(gp_Ax3(),0.3,2.0);
  else if(kind==1)surface=new Geom_CylindricalSurface(gp_Ax3(),2.0);
  else {NCollection_Array2<gp_Pnt> poles(1,2,1,2);for(int r=1;r<=2;++r)for(int c=1;c<=2;++c)poles(r,c)=gp_Pnt((r-1)*2.0,(c-1)*3.0,0.0);surface=new Geom_BezierSurface(poles);}
  TopoDS_Face face;BRep_Builder().MakeFace(face,surface,tolerance);
  occ::handle<NCollection_IncAllocator> allocator=new NCollection_IncAllocator;
  return new (allocator) BRepMeshData_Face(face,allocator);
}
void mesh_range(std::istream& input) {
  int kind,bound,tol,adjust;input>>kind>>bound>>tol>>adjust;
  IMeshTools_Parameters parameters;parameters.MinSize=0.001;parameters.Angle=0.5;
  auto face=range_face(kind,tol?0.01:1e-7);face->SetDeflection(tol?0.1:0.01);
  BRepMesh_ConeRangeSplitter splitter;ga_flag(splitter.IsValid());splitter.Reset(face,parameters);
  const double bounds[][4]={{0,1,0,1},{-1,8,-2,3},{.2,.7,.1,.8},{0,0,0,1},{0,1,-9,-7},{0,.001,0,.002}};
  splitter.AddPoint(gp_Pnt2d(bounds[bound][0],bounds[bound][2]));splitter.AddPoint(gp_Pnt2d(bounds[bound][1],bounds[bound][3]));
  if(adjust)splitter.AdjustRange();ga_flag(splitter.IsValid());
  for(auto pair:{splitter.GetRangeU(),splitter.GetRangeV(),splitter.GetDelta(),splitter.GetToleranceUV()}){ga_real(pair.first);ga_real(pair.second);}
  gp_Pnt2d p(.37,.62);ga_pnt2d(splitter.Scale(p,true));ga_pnt2d(splitter.Scale(p,false));ga_pnt(splitter.Point(p));
  ga_flag(!splitter.GetDFace().IsNull());ga_flag(!splitter.GetSurface().IsNull());
  if(kind==0&&bound!=3){std::pair<int,int>counts;auto steps=splitter.GetSplitSteps(parameters,counts);ga_real(steps.first);ga_real(steps.second);ga_int(counts.first);ga_int(counts.second);auto nodes=splitter.GenerateSurfaceNodes(parameters);ga_int(nodes->Size());for(const auto& node:*nodes)ga_pnt2d(node);}
  ga_flag(splitter.BRepMesh_DefaultRangeSplitter::GenerateSurfaceNodes(parameters).IsNull());
  splitter.Reset(face,parameters);ga_flag(splitter.IsValid());ga_end();
}
