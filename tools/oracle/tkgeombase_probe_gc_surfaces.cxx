// TKGeomBase oracle: the GC surface and transformation algorithms (tkgeombase_probe_gc.cxx).
// gp_Pln, gp_Cone and gp_Cylinder inputs are read as a gp_Ax3 then their parameters.
// gc_plane k: 0 (Pln) | 1 (P, V) | 2 (A, B, C, D) | 3 (P1, P2, P3) | 4 (Pln, Dist) | 5 (Pln, P) | 6 (Ax1).
void gb_gc_plane_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax3 a=r.ax3();GB_GC(gb_geom_surface,GC_MakePlane(gp_Pln(a)));}
 if(k==1){gp_Pnt p=r.pnt();gp_Dir d=r.dir();GB_GC(gb_geom_surface,GC_MakePlane(p,d));}
 if(k==2){double a=r.real(),b=r.real(),c=r.real(),d=r.real();GB_GC(gb_geom_surface,GC_MakePlane(a,b,c,d));}
 if(k==3){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GC(gb_geom_surface,GC_MakePlane(a,b,c));}
 if(k==4){gp_Ax3 a=r.ax3();double d=r.real();GB_GC(gb_geom_surface,GC_MakePlane(gp_Pln(a),d));}
 if(k==5){gp_Ax3 a=r.ax3();gp_Pnt p=r.pnt();GB_GC(gb_geom_surface,GC_MakePlane(gp_Pln(a),p));}
 if(k==6){gp_Ax1 a=r.ax1();GB_GC(gb_geom_surface,GC_MakePlane(a));}
 gb_end();
}
// gc_conical k: 0 (A2, Ang, R) | 1 (Cone(Ax3, Ang, R)) | 2 (P1, P2, P3, P4) | 3 (P1, P2, R1, R2).
void gb_gc_conical_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2 a=r.ax2();double g=r.real(),d=r.real();GB_GC(gb_geom_surface,GC_MakeConicalSurface(a,g,d));}
 if(k==1){gp_Ax3 a=r.ax3();double g=r.real(),d=r.real();GB_GC(gb_geom_surface,GC_MakeConicalSurface(gp_Cone(a,g,d)));}
 if(k==2){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt(),d=r.pnt();GB_GC(gb_geom_surface,GC_MakeConicalSurface(a,b,c,d));}
 if(k==3){gp_Pnt a=r.pnt(),b=r.pnt();double r1=r.real(),r2=r.real();GB_GC(gb_geom_surface,GC_MakeConicalSurface(a,b,r1,r2));}
 gb_end();
}
// gc_cylindrical k: 0 (Cyl(Ax3, R)) | 1 (A2, R) | 2 (A1, R) | 3 (Circ) | 4 (P1, P2, P3) |
// 5 (Cyl, Dist) | 6 (Cyl, P).
void gb_gc_cylindrical_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax3 a=r.ax3();double d=r.real();GB_GC(gb_geom_surface,GC_MakeCylindricalSurface(gp_Cylinder(a,d)));}
 if(k==1){gp_Ax2 a=r.ax2();double d=r.real();GB_GC(gb_geom_surface,GC_MakeCylindricalSurface(a,d));}
 if(k==2){gp_Ax1 a=r.ax1();double d=r.real();GB_GC(gb_geom_surface,GC_MakeCylindricalSurface(a,d));}
 if(k==3){gp_Circ c=gb_read_circ(r);GB_GC(gb_geom_surface,GC_MakeCylindricalSurface(c));}
 if(k==4){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GC(gb_geom_surface,GC_MakeCylindricalSurface(a,b,c));}
 if(k==5){gp_Ax3 a=r.ax3();double r0=r.real(),d=r.real();GB_GC(gb_geom_surface,GC_MakeCylindricalSurface(gp_Cylinder(a,r0),d));}
 if(k==6){gp_Ax3 a=r.ax3();double r0=r.real();gp_Pnt p=r.pnt();GB_GC(gb_geom_surface,GC_MakeCylindricalSurface(gp_Cylinder(a,r0),p));}
 gb_end();
}
// gc_trimmed_cone k: 0 (P1, P2, P3, P4) | 1 (P1, P2, R1, R2);
// gc_trimmed_cylinder k: 0 (P1, P2, P3) | 1 (Circ, H) | 2 (A1, R, H).
void gb_gc_trimmed_cone_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt(),d=r.pnt();GB_GC(gb_geom_surface,GC_MakeTrimmedCone(a,b,c,d));}
 if(k==1){gp_Pnt a=r.pnt(),b=r.pnt();double r1=r.real(),r2=r.real();GB_GC(gb_geom_surface,GC_MakeTrimmedCone(a,b,r1,r2));}
 gb_end();
}
void gb_gc_trimmed_cylinder_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GC(gb_geom_surface,GC_MakeTrimmedCylinder(a,b,c));}
 if(k==1){gp_Circ c=gb_read_circ(r);double h=r.real();GB_GC(gb_geom_surface,GC_MakeTrimmedCylinder(c,h));}
 if(k==2){gp_Ax1 a=r.ax1();double d=r.real(),h=r.real();GB_GC(gb_geom_surface,GC_MakeTrimmedCylinder(a,d,h));}
 gb_end();
}
// gc_mirror k: 0 (P) | 1 (Ax1) | 2 (Lin(Ax1)) | 3 (P, D) | 4 (Pln(Ax3)) | 5 (Ax2);
// gc_mirror2d k: 0 (P) | 1 (Ax2d) | 2 (Lin(Ax2d)) | 3 (P, D).
void gb_gc_mirror_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Pnt p=r.pnt();GB_GC_TRSF(gb_geom_trsf,GC_MakeMirror(p));}
 if(k==1){gp_Ax1 a=r.ax1();GB_GC_TRSF(gb_geom_trsf,GC_MakeMirror(a));}
 if(k==2){gp_Ax1 a=r.ax1();GB_GC_TRSF(gb_geom_trsf,GC_MakeMirror(gp_Lin(a)));}
 if(k==3){gp_Pnt p=r.pnt();gp_Dir d=r.dir();GB_GC_TRSF(gb_geom_trsf,GC_MakeMirror(p,d));}
 if(k==4){gp_Ax3 a=r.ax3();GB_GC_TRSF(gb_geom_trsf,GC_MakeMirror(gp_Pln(a)));}
 if(k==5){gp_Ax2 a=r.ax2();GB_GC_TRSF(gb_geom_trsf,GC_MakeMirror(a));}
 gb_end();
}
void gb_gc_mirror2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Pnt2d p=r.pnt2d();GB_GC_TRSF(gb_geom2d_trsf,GC_MakeMirror2d(p));}
 if(k==1){gp_Ax2d a=r.ax2d();GB_GC_TRSF(gb_geom2d_trsf,GC_MakeMirror2d(a));}
 if(k==2){gp_Ax2d a=r.ax2d();GB_GC_TRSF(gb_geom2d_trsf,GC_MakeMirror2d(gp_Lin2d(a)));}
 if(k==3){gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();GB_GC_TRSF(gb_geom2d_trsf,GC_MakeMirror2d(p,d));}
 gb_end();
}
// gc_rotation k: 0 (Lin(Ax1), Angle) | 1 (Ax1, Angle) | 2 (P, D, Angle); gc_rotation2d (P, Angle);
// gc_scale (P, S); gc_scale2d (P, S); gc_translation(2d) k: 0 (Vec) | 1 (P1, P2).
void gb_gc_rotation_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax1 a=r.ax1();double g=r.real();GB_GC_TRSF(gb_geom_trsf,GC_MakeRotation(gp_Lin(a),g));}
 if(k==1){gp_Ax1 a=r.ax1();double g=r.real();GB_GC_TRSF(gb_geom_trsf,GC_MakeRotation(a,g));}
 if(k==2){gp_Pnt p=r.pnt();gp_Dir d=r.dir();double g=r.real();GB_GC_TRSF(gb_geom_trsf,GC_MakeRotation(p,d,g));}
 gb_end();
}
void gb_gc_rotation2d_case(std::istream& in){
 GbReader r{in};gp_Pnt2d p=r.pnt2d();double g=r.real();GB_GC_TRSF(gb_geom2d_trsf,GC_MakeRotation2d(p,g));gb_end();
}
void gb_gc_scale_case(std::istream& in){
 GbReader r{in};gp_Pnt p=r.pnt();double s=r.real();GB_GC_TRSF(gb_geom_trsf,GC_MakeScale(p,s));gb_end();
}
void gb_gc_scale2d_case(std::istream& in){
 GbReader r{in};gp_Pnt2d p=r.pnt2d();double s=r.real();GB_GC_TRSF(gb_geom2d_trsf,GC_MakeScale2d(p,s));gb_end();
}
void gb_gc_translation_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Vec v=r.vec();GB_GC_TRSF(gb_geom_trsf,GC_MakeTranslation(v));}
 if(k==1){gp_Pnt a=r.pnt(),b=r.pnt();GB_GC_TRSF(gb_geom_trsf,GC_MakeTranslation(a,b));}
 gb_end();
}
void gb_gc_translation2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Vec2d v=r.vec2d();GB_GC_TRSF(gb_geom2d_trsf,GC_MakeTranslation2d(v));}
 if(k==1){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();GB_GC_TRSF(gb_geom2d_trsf,GC_MakeTranslation2d(a,b));}
 gb_end();
}
