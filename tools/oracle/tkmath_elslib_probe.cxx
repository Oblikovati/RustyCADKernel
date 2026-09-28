// ElSLib static functions and gp-object overloads (included by tkmath_elementary_probe.cxx).
struct ElSurfaceRequest{gp_Ax3 frame;double u,v,r,a,major,minor;int nu,nv;gp_Pnt point;};
void el_d2(const gp_Pnt& p,const gp_Vec& du,const gp_Vec& dv,const gp_Vec& duu,const gp_Vec& dvv,const gp_Vec& duv){el_pnt(p);el_vec(du);el_vec(dv);el_vec(duu);el_vec(dvv);el_vec(duv);}
void el_uv(double u,double v){el_real(u);el_real(v);}
void el_plane_functions(const ElSurfaceRequest& q){
 gp_Pnt p;gp_Vec du,dv;double u,v;
 el_pnt(ElSLib::PlaneValue(q.u,q.v,q.frame));ElSLib::PlaneD0(q.u,q.v,q.frame,p);el_pnt(p);ElSLib::PlaneD1(q.u,q.v,q.frame,p,du,dv);el_pnt(p);el_vec(du);el_vec(dv);
 el_vec(ElSLib::PlaneDN(q.u,q.v,q.frame,q.nu,q.nv));ElSLib::PlaneParameters(q.frame,q.point,u,v);el_uv(u,v);
 el_lin(ElSLib::PlaneUIso(q.frame,q.u));el_lin(ElSLib::PlaneVIso(q.frame,q.v));
}
void el_cylinder_functions(const ElSurfaceRequest& q){
 gp_Pnt p;gp_Vec du,dv,duu,dvv,duv,d3a,d3b,d3c,d3d;double u,v;
 el_pnt(ElSLib::CylinderValue(q.u,q.v,q.frame,q.r));ElSLib::CylinderD0(q.u,q.v,q.frame,q.r,p);el_pnt(p);ElSLib::CylinderD1(q.u,q.v,q.frame,q.r,p,du,dv);el_pnt(p);el_vec(du);el_vec(dv);
 ElSLib::CylinderD2(q.u,q.v,q.frame,q.r,p,du,dv,duu,dvv,duv);el_d2(p,du,dv,duu,dvv,duv);
 ElSLib::CylinderD3(q.u,q.v,q.frame,q.r,p,du,dv,duu,dvv,duv,d3a,d3b,d3c,d3d);el_d2(p,du,dv,duu,dvv,duv);el_vec(d3a);el_vec(d3b);el_vec(d3c);el_vec(d3d);
 el_vec(ElSLib::CylinderDN(q.u,q.v,q.frame,q.r,q.nu,q.nv));ElSLib::CylinderParameters(q.frame,q.r,q.point,u,v);el_uv(u,v);
 el_lin(ElSLib::CylinderUIso(q.frame,q.r,q.u));el_circ(ElSLib::CylinderVIso(q.frame,q.r,q.v));
}
void el_cone_functions(const ElSurfaceRequest& q){
 gp_Pnt p;gp_Vec du,dv,duu,dvv,duv,d3a,d3b,d3c,d3d;double u,v;
 el_pnt(ElSLib::ConeValue(q.u,q.v,q.frame,q.r,q.a));ElSLib::ConeD0(q.u,q.v,q.frame,q.r,q.a,p);el_pnt(p);ElSLib::ConeD1(q.u,q.v,q.frame,q.r,q.a,p,du,dv);el_pnt(p);el_vec(du);el_vec(dv);
 ElSLib::ConeD2(q.u,q.v,q.frame,q.r,q.a,p,du,dv,duu,dvv,duv);el_d2(p,du,dv,duu,dvv,duv);
 ElSLib::ConeD3(q.u,q.v,q.frame,q.r,q.a,p,du,dv,duu,dvv,duv,d3a,d3b,d3c,d3d);el_d2(p,du,dv,duu,dvv,duv);el_vec(d3a);el_vec(d3b);el_vec(d3c);el_vec(d3d);
 el_vec(ElSLib::ConeDN(q.u,q.v,q.frame,q.r,q.a,q.nu,q.nv));ElSLib::ConeParameters(q.frame,q.r,q.a,q.point,u,v);el_uv(u,v);
 el_lin(ElSLib::ConeUIso(q.frame,q.r,q.a,q.u));el_circ(ElSLib::ConeVIso(q.frame,q.r,q.a,q.v));
}
void el_sphere_functions(const ElSurfaceRequest& q){
 gp_Pnt p;gp_Vec du,dv,duu,dvv,duv,d3a,d3b,d3c,d3d;double u,v;
 el_pnt(ElSLib::SphereValue(q.u,q.v,q.frame,q.r));ElSLib::SphereD0(q.u,q.v,q.frame,q.r,p);el_pnt(p);ElSLib::SphereD1(q.u,q.v,q.frame,q.r,p,du,dv);el_pnt(p);el_vec(du);el_vec(dv);
 ElSLib::SphereD2(q.u,q.v,q.frame,q.r,p,du,dv,duu,dvv,duv);el_d2(p,du,dv,duu,dvv,duv);
 ElSLib::SphereD3(q.u,q.v,q.frame,q.r,p,du,dv,duu,dvv,duv,d3a,d3b,d3c,d3d);el_d2(p,du,dv,duu,dvv,duv);el_vec(d3a);el_vec(d3b);el_vec(d3c);el_vec(d3d);
 el_vec(ElSLib::SphereDN(q.u,q.v,q.frame,q.r,q.nu,q.nv));ElSLib::SphereParameters(q.frame,q.r,q.point,u,v);el_uv(u,v);
 el_circ(ElSLib::SphereUIso(q.frame,q.r,q.u));el_circ(ElSLib::SphereVIso(q.frame,q.r,q.v));
}
void el_torus_functions(const ElSurfaceRequest& q){
 gp_Pnt p;gp_Vec du,dv,duu,dvv,duv,d3a,d3b,d3c,d3d;double u,v;
 el_pnt(ElSLib::TorusValue(q.u,q.v,q.frame,q.major,q.minor));ElSLib::TorusD0(q.u,q.v,q.frame,q.major,q.minor,p);el_pnt(p);
 ElSLib::TorusD1(q.u,q.v,q.frame,q.major,q.minor,p,du,dv);el_pnt(p);el_vec(du);el_vec(dv);
 ElSLib::TorusD2(q.u,q.v,q.frame,q.major,q.minor,p,du,dv,duu,dvv,duv);el_d2(p,du,dv,duu,dvv,duv);
 ElSLib::TorusD3(q.u,q.v,q.frame,q.major,q.minor,p,du,dv,duu,dvv,duv,d3a,d3b,d3c,d3d);el_d2(p,du,dv,duu,dvv,duv);el_vec(d3a);el_vec(d3b);el_vec(d3c);el_vec(d3d);
 el_vec(ElSLib::TorusDN(q.u,q.v,q.frame,q.major,q.minor,q.nu,q.nv));ElSLib::TorusParameters(q.frame,q.major,q.minor,q.point,u,v);el_uv(u,v);
 el_circ(ElSLib::TorusUIso(q.frame,q.major,q.minor,q.u));el_circ(ElSLib::TorusVIso(q.frame,q.major,q.minor,q.v));
}
template<class Surface> void el_surface_overloads(const Surface& s,const ElSurfaceRequest& q){
 gp_Pnt p;gp_Vec du,dv;double u,v;
 el_pnt(ElSLib::Value(q.u,q.v,s));ElSLib::D0(q.u,q.v,s,p);el_pnt(p);ElSLib::D1(q.u,q.v,s,p,du,dv);el_vec(dv);el_vec(ElSLib::DN(q.u,q.v,s,q.nu,q.nv));
 ElSLib::Parameters(s,q.point,u,v);el_uv(u,v);
}
template<class Surface> void el_curved_overloads(const Surface& s,const ElSurfaceRequest& q){
 gp_Pnt p;gp_Vec du,dv,duu,dvv,duv,d3a,d3b,d3c,d3d;el_surface_overloads(s,q);
 ElSLib::D2(q.u,q.v,s,p,du,dv,duu,dvv,duv);el_vec(duv);ElSLib::D3(q.u,q.v,s,p,du,dv,duu,dvv,duv,d3a,d3b,d3c,d3d);el_vec(d3d);
}
void el_overload_surfaces(const ElSurfaceRequest& q){
 el_surface_overloads(gp_Pln(q.frame),q);
 try{el_curved_overloads(gp_Cylinder(q.frame,q.r),q);}catch(const Standard_Failure& e){el_fail(e);}
 try{el_curved_overloads(gp_Cone(q.frame,q.a,q.r),q);}catch(const Standard_Failure& e){el_fail(e);}
 try{el_curved_overloads(gp_Sphere(q.frame,q.r),q);}catch(const Standard_Failure& e){el_fail(e);}
 try{el_curved_overloads(gp_Torus(q.frame,q.major,q.minor),q);}catch(const Standard_Failure& e){el_fail(e);}
}
void elslib_case(std::istream& in){
 gp_Pnt O=el_read_pnt(in);gp_XYZ N=el_read_xyz(in),X=el_read_xyz(in);int indirect;in>>indirect;
 ElSurfaceRequest q;in>>q.u>>q.v>>q.r>>q.a>>q.major>>q.minor>>q.nu>>q.nv;q.point=el_read_pnt(in);
 try{q.frame=gp_Ax3(O,gp_Dir(N),gp_Dir(X));}catch(const Standard_Failure& e){el_fail(e);std::cout<<'\n';return;}
 if(indirect)q.frame.YReverse();
 el_ax3(q.frame);el_plane_functions(q);el_cylinder_functions(q);el_cone_functions(q);el_sphere_functions(q);el_torus_functions(q);el_overload_surfaces(q);
 std::cout<<'\n';
}
