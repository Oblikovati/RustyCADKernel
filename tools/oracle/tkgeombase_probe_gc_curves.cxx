// TKGeomBase oracle: the GC curve algorithms (tkgeombase_probe_gc.cxx). gp inputs are read
// as their frame then their parameters: circle F R, ellipse and hyperbola F Major Minor,
// parabola F Focal (F a gp_Ax2 in 3D, a gp_Ax22d in 2D), line gp_Ax1 or gp_Ax2d.
gp_Circ gb_read_circ(GbReader& r){gp_Ax2 f=r.ax2();return gp_Circ(f,r.real());}
gp_Elips gb_read_elips(GbReader& r){gp_Ax2 f=r.ax2();double a=r.real();return gp_Elips(f,a,r.real());}
gp_Hypr gb_read_hypr(GbReader& r){gp_Ax2 f=r.ax2();double a=r.real();return gp_Hypr(f,a,r.real());}
gp_Parab gb_read_parab(GbReader& r){gp_Ax2 f=r.ax2();return gp_Parab(f,r.real());}
gp_Circ2d gb_read_circ2d(GbReader& r){gp_Ax22d f=r.ax22d();return gp_Circ2d(f,r.real());}
gp_Elips2d gb_read_elips2d(GbReader& r){gp_Ax22d f=r.ax22d();double a=r.real();return gp_Elips2d(f,a,r.real());}
gp_Hypr2d gb_read_hypr2d(GbReader& r){gp_Ax22d f=r.ax22d();double a=r.real();return gp_Hypr2d(f,a,r.real());}
gp_Parab2d gb_read_parab2d(GbReader& r){gp_Ax22d f=r.ax22d();return gp_Parab2d(f,r.real());}
// The three shared arc overloads: k 0 (Conic, A1, A2, Sense) | 1 (Conic, P, Alpha, Sense) |
// 2 (Conic, P1, P2, Sense); `read` reads the conic, `point` a point.
template<class M,class C,class P> void gb_gc_arc(int k,GbReader& r,C conic,P(GbReader::*point)()){
 if(k==0){double a=r.real(),b=r.real();bool s=r.flag();GB_GC(gb_geom_curve,M(conic,a,b,s));}
 if(k==1){P p=(r.*point)();double a=r.real();bool s=r.flag();GB_GC(gb_geom_curve,M(conic,p,a,s));}
 if(k==2){P p=(r.*point)(),q=(r.*point)();bool s=r.flag();GB_GC(gb_geom_curve,M(conic,p,q,s));}
}
template<class M,class C,class P> void gb_gc_arc2d(int k,GbReader& r,C conic,P(GbReader::*point)()){
 if(k==0){double a=r.real(),b=r.real();bool s=r.flag();GB_GC(gb_geom2d_curve,M(conic,a,b,s));}
 if(k==1){P p=(r.*point)();double a=r.real();bool s=r.flag();GB_GC(gb_geom2d_curve,M(conic,p,a,s));}
 if(k==2){P p=(r.*point)(),q=(r.*point)();bool s=r.flag();GB_GC(gb_geom2d_curve,M(conic,p,q,s));}
}
// gc_arc_circle k: 0-2 the shared overloads on a circle | 3 (P1, P2, P3) | 4 (P1, V, P2).
void gb_gc_arc_circle_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k<=2)gb_gc_arc<GC_MakeArcOfCircle>(k,r,gb_read_circ(r),&GbReader::pnt);
 if(k==3){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GC(gb_geom_curve,GC_MakeArcOfCircle(a,b,c));}
 if(k==4){gp_Pnt a=r.pnt();gp_Vec v=r.vec();gp_Pnt b=r.pnt();GB_GC(gb_geom_curve,GC_MakeArcOfCircle(a,v,b));}
 gb_end();
}
void gb_gc_arc_ellipse_case(std::istream& in){
 GbReader r{in};int k=r.integer();gb_gc_arc<GC_MakeArcOfEllipse>(k,r,gb_read_elips(r),&GbReader::pnt);gb_end();
}
void gb_gc_arc_hyperbola_case(std::istream& in){
 GbReader r{in};int k=r.integer();gb_gc_arc<GC_MakeArcOfHyperbola>(k,r,gb_read_hypr(r),&GbReader::pnt);gb_end();
}
void gb_gc_arc_parabola_case(std::istream& in){
 GbReader r{in};int k=r.integer();gb_gc_arc<GC_MakeArcOfParabola>(k,r,gb_read_parab(r),&GbReader::pnt);gb_end();
}
// gc_arc_circle2d k: 0-2 the shared overloads on a circle | 3 (P1, P2, P3) | 4 (P1, V, P2).
void gb_gc_arc_circle2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k<=2)gb_gc_arc2d<GC_MakeArcOfCircle2d>(k,r,gb_read_circ2d(r),&GbReader::pnt2d);
 if(k==3){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d(),c=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeArcOfCircle2d(a,b,c));}
 if(k==4){gp_Pnt2d a=r.pnt2d();gp_Vec2d v=r.vec2d();gp_Pnt2d b=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeArcOfCircle2d(a,v,b));}
 gb_end();
}
void gb_gc_arc_ellipse2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();gb_gc_arc2d<GC_MakeArcOfEllipse2d>(k,r,gb_read_elips2d(r),&GbReader::pnt2d);gb_end();
}
void gb_gc_arc_hyperbola2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();gb_gc_arc2d<GC_MakeArcOfHyperbola2d>(k,r,gb_read_hypr2d(r),&GbReader::pnt2d);gb_end();
}
void gb_gc_arc_parabola2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();gb_gc_arc2d<GC_MakeArcOfParabola2d>(k,r,gb_read_parab2d(r),&GbReader::pnt2d);gb_end();
}
// gc_circle k: 0 (Circ) | 1 (A2, R) | 2 (Circ, Dist) | 3 (Circ, P) | 4 (P1, P2, P3) |
// 5 (P, Norm, R) | 6 (P, PtAxis, R) | 7 (Ax1, R).
void gb_gc_circle_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Circ c=gb_read_circ(r);GB_GC(gb_geom_curve,GC_MakeCircle(c));}
 if(k==1){gp_Ax2 a=r.ax2();double d=r.real();GB_GC(gb_geom_curve,GC_MakeCircle(a,d));}
 if(k==2){gp_Circ c=gb_read_circ(r);double d=r.real();GB_GC(gb_geom_curve,GC_MakeCircle(c,d));}
 if(k==3){gp_Circ c=gb_read_circ(r);gp_Pnt p=r.pnt();GB_GC(gb_geom_curve,GC_MakeCircle(c,p));}
 if(k==4){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GC(gb_geom_curve,GC_MakeCircle(a,b,c));}
 if(k==5){gp_Pnt c=r.pnt();gp_Dir n=r.dir();double d=r.real();GB_GC(gb_geom_curve,GC_MakeCircle(c,n,d));}
 if(k==6){gp_Pnt c=r.pnt(),p=r.pnt();double d=r.real();GB_GC(gb_geom_curve,GC_MakeCircle(c,p,d));}
 if(k==7){gp_Ax1 a=r.ax1();double d=r.real();GB_GC(gb_geom_curve,GC_MakeCircle(a,d));}
 gb_end();
}
// gc_circle2d k: 0 (Circ) | 1 (Ax2d, R, Sense) | 2 (Ax22d, R) | 3 (Circ, Dist) | 4 (Circ, P) |
// 5 (P1, P2, P3) | 6 (P, R, Sense) | 7 (Center, P, Sense).
void gb_gc_circle2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Circ2d c=gb_read_circ2d(r);GB_GC(gb_geom2d_curve,GC_MakeCircle2d(c));}
 if(k==1){gp_Ax2d a=r.ax2d();double d=r.real();bool s=r.flag();GB_GC(gb_geom2d_curve,GC_MakeCircle2d(a,d,s));}
 if(k==2){gp_Ax22d a=r.ax22d();double d=r.real();GB_GC(gb_geom2d_curve,GC_MakeCircle2d(a,d));}
 if(k==3){gp_Circ2d c=gb_read_circ2d(r);double d=r.real();GB_GC(gb_geom2d_curve,GC_MakeCircle2d(c,d));}
 if(k==4){gp_Circ2d c=gb_read_circ2d(r);gp_Pnt2d p=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeCircle2d(c,p));}
 if(k==5){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d(),c=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeCircle2d(a,b,c));}
 if(k==6){gp_Pnt2d c=r.pnt2d();double d=r.real();bool s=r.flag();GB_GC(gb_geom2d_curve,GC_MakeCircle2d(c,d,s));}
 if(k==7){gp_Pnt2d c=r.pnt2d(),p=r.pnt2d();bool s=r.flag();GB_GC(gb_geom2d_curve,GC_MakeCircle2d(c,p,s));}
 gb_end();
}
// gc_ellipse and gc_hyperbola k: 0 (gp conic) | 1 (A2, Major, Minor) | 2 (S1, S2, Center).
void gb_gc_ellipse_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Elips e=gb_read_elips(r);GB_GC(gb_geom_curve,GC_MakeEllipse(e));}
 if(k==1){gp_Ax2 a=r.ax2();double m=r.real(),n=r.real();GB_GC(gb_geom_curve,GC_MakeEllipse(a,m,n));}
 if(k==2){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GC(gb_geom_curve,GC_MakeEllipse(a,b,c));}
 gb_end();
}
void gb_gc_hyperbola_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Hypr e=gb_read_hypr(r);GB_GC(gb_geom_curve,GC_MakeHyperbola(e));}
 if(k==1){gp_Ax2 a=r.ax2();double m=r.real(),n=r.real();GB_GC(gb_geom_curve,GC_MakeHyperbola(a,m,n));}
 if(k==2){gp_Pnt a=r.pnt(),b=r.pnt(),c=r.pnt();GB_GC(gb_geom_curve,GC_MakeHyperbola(a,b,c));}
 gb_end();
}
// gc_ellipse2d k: 0 (gp) | 1 (Ax22d, Major, Minor) | 2 (Ax2d, Major, Minor, Sense) | 3 (S1, S2, C);
// gc_hyperbola2d k: 0 (gp) | 1 (Ax2d, Major, Minor, Sense) | 2 (Ax22d, Major, Minor) | 3 (S1, S2, C).
void gb_gc_ellipse2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Elips2d e=gb_read_elips2d(r);GB_GC(gb_geom2d_curve,GC_MakeEllipse2d(e));}
 if(k==1){gp_Ax22d a=r.ax22d();double m=r.real(),n=r.real();GB_GC(gb_geom2d_curve,GC_MakeEllipse2d(a,m,n));}
 if(k==2){gp_Ax2d a=r.ax2d();double m=r.real(),n=r.real();bool s=r.flag();GB_GC(gb_geom2d_curve,GC_MakeEllipse2d(a,m,n,s));}
 if(k==3){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d(),c=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeEllipse2d(a,b,c));}
 gb_end();
}
void gb_gc_hyperbola2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Hypr2d e=gb_read_hypr2d(r);GB_GC(gb_geom2d_curve,GC_MakeHyperbola2d(e));}
 if(k==1){gp_Ax2d a=r.ax2d();double m=r.real(),n=r.real();bool s=r.flag();GB_GC(gb_geom2d_curve,GC_MakeHyperbola2d(a,m,n,s));}
 if(k==2){gp_Ax22d a=r.ax22d();double m=r.real(),n=r.real();GB_GC(gb_geom2d_curve,GC_MakeHyperbola2d(a,m,n));}
 if(k==3){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d(),c=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeHyperbola2d(a,b,c));}
 gb_end();
}
// gc_parabola2d k: 0 (gp) | 1 (MirrorAxis, Focal, Sense) | 2 (Ax22d, Focal) | 3 (D, F, Sense) | 4 (S1, O).
void gb_gc_parabola2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Parab2d p=gb_read_parab2d(r);GB_GC(gb_geom2d_curve,GC_MakeParabola2d(p));}
 if(k==1){gp_Ax2d a=r.ax2d();double f=r.real();bool s=r.flag();GB_GC(gb_geom2d_curve,GC_MakeParabola2d(a,f,s));}
 if(k==2){gp_Ax22d a=r.ax22d();double f=r.real();GB_GC(gb_geom2d_curve,GC_MakeParabola2d(a,f));}
 if(k==3){gp_Ax2d a=r.ax2d();gp_Pnt2d f=r.pnt2d();bool s=r.flag();GB_GC(gb_geom2d_curve,GC_MakeParabola2d(a,f,s));}
 if(k==4){gp_Pnt2d a=r.pnt2d(),o=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeParabola2d(a,o));}
 gb_end();
}
// gc_line k: 0 (P, V) | 1 (Ax1) | 2 (Lin) | 3 (P1, P2) | 4 (Lin, P).
void gb_gc_line_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Pnt p=r.pnt();gp_Dir d=r.dir();GB_GC(gb_geom_curve,GC_MakeLine(p,d));}
 if(k==1){gp_Ax1 a=r.ax1();GB_GC(gb_geom_curve,GC_MakeLine(a));}
 if(k==2){gp_Ax1 a=r.ax1();GB_GC(gb_geom_curve,GC_MakeLine(gp_Lin(a)));}
 if(k==3){gp_Pnt a=r.pnt(),b=r.pnt();GB_GC(gb_geom_curve,GC_MakeLine(a,b));}
 if(k==4){gp_Ax1 a=r.ax1();gp_Pnt p=r.pnt();GB_GC(gb_geom_curve,GC_MakeLine(gp_Lin(a),p));}
 gb_end();
}
// gc_line2d k: 0 (Ax2d) | 1 (Lin) | 2 (P, V) | 3 (P1, P2) | 4 (Lin, P) | 5 (Lin, Dist).
void gb_gc_line2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Ax2d a=r.ax2d();GB_GC(gb_geom2d_curve,GC_MakeLine2d(a));}
 if(k==1){gp_Ax2d a=r.ax2d();GB_GC(gb_geom2d_curve,GC_MakeLine2d(gp_Lin2d(a)));}
 if(k==2){gp_Pnt2d p=r.pnt2d();gp_Dir2d d=r.dir2d();GB_GC(gb_geom2d_curve,GC_MakeLine2d(p,d));}
 if(k==3){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeLine2d(a,b));}
 if(k==4){gp_Ax2d a=r.ax2d();gp_Pnt2d p=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeLine2d(gp_Lin2d(a),p));}
 if(k==5){gp_Ax2d a=r.ax2d();double d=r.real();GB_GC(gb_geom2d_curve,GC_MakeLine2d(gp_Lin2d(a),d));}
 gb_end();
}
// gc_segment k: 0 (P1, P2) | 1 (Lin, P, U) | 2 (Lin, P1, P2) | 3 (Lin, U1, U2).
void gb_gc_segment_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Pnt a=r.pnt(),b=r.pnt();GB_GC(gb_geom_curve,GC_MakeSegment(a,b));}
 if(k==1){gp_Ax1 a=r.ax1();gp_Pnt p=r.pnt();double u=r.real();GB_GC(gb_geom_curve,GC_MakeSegment(gp_Lin(a),p,u));}
 if(k==2){gp_Ax1 a=r.ax1();gp_Pnt p=r.pnt(),q=r.pnt();GB_GC(gb_geom_curve,GC_MakeSegment(gp_Lin(a),p,q));}
 if(k==3){gp_Ax1 a=r.ax1();double u=r.real(),v=r.real();GB_GC(gb_geom_curve,GC_MakeSegment(gp_Lin(a),u,v));}
 gb_end();
}
// gc_segment2d k: 0 (P1, V, P2) | 1 (P1, P2) | 2 (Lin, P, U) | 3 (Lin, P1, P2) | 4 (Lin, U1, U2).
void gb_gc_segment2d_case(std::istream& in){
 GbReader r{in};int k=r.integer();
 if(k==0){gp_Pnt2d a=r.pnt2d();gp_Dir2d d=r.dir2d();gp_Pnt2d b=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeSegment2d(a,d,b));}
 if(k==1){gp_Pnt2d a=r.pnt2d(),b=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeSegment2d(a,b));}
 if(k==2){gp_Ax2d a=r.ax2d();gp_Pnt2d p=r.pnt2d();double u=r.real();GB_GC(gb_geom2d_curve,GC_MakeSegment2d(gp_Lin2d(a),p,u));}
 if(k==3){gp_Ax2d a=r.ax2d();gp_Pnt2d p=r.pnt2d(),q=r.pnt2d();GB_GC(gb_geom2d_curve,GC_MakeSegment2d(gp_Lin2d(a),p,q));}
 if(k==4){gp_Ax2d a=r.ax2d();double u=r.real(),v=r.real();GB_GC(gb_geom2d_curve,GC_MakeSegment2d(gp_Lin2d(a),u,v));}
 gb_end();
}
