// Native real mesh and named driver output shared by isoline overload proofs.
#pragma once
static occ::handle<Poly_Triangulation> isoline_grid_mesh(const occ::handle<Geom_Surface>& surface,int order) {
  const occ::handle<Poly_Triangulation> mesh=new Poly_Triangulation(9,8,true);
  for(int j=0;j<3;++j)for(int i=0;i<3;++i) {
    const gp_Pnt2d uv(i*.5,j*.5);gp_Pnt xyz=surface.IsNull()?gp_Pnt(uv.X(),uv.Y(),0):surface->Value(uv.X(),uv.Y());
    xyz.SetZ(xyz.Z()+.1*(i+j));mesh->SetNode(j*3+i+1,xyz);mesh->SetUVNode(j*3+i+1,uv);
  }
  std::vector<Poly_Triangle> triangles;
  for(int j=0;j<2;++j)for(int i=0;i<2;++i){const int a=j*3+i+1;triangles.emplace_back(a,a+1,a+4);triangles.emplace_back(a,a+4,a+3);}
  if(order==1)std::reverse(triangles.begin(),triangles.end());
  if(order==2)std::rotate(triangles.begin(),triangles.begin()+3,triangles.end());
  for(int i=0;i<8;++i)mesh->SetTriangle(i+1,triangles[i]);
  return mesh;
}
static void print_isoline_presentation(const occ::handle<Prs3d_Presentation>& prs,const XrProbeEvents& events,int scenario,bool failed) {
  std::cout<<scenario<<' '<<failed<<' '<<prs->Groups().Length();
  for(const auto& group:prs->Groups())XrPrintGroup(group);
  std::cout<<' '<<events.callbacks.size();for(int call:events.callbacks)std::cout<<' '<<call;std::cout<<'\n';
}
