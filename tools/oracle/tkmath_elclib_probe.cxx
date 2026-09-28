// ElCLib static functions and gp-object overloads (included by tkmath_elementary_probe.cxx).
void el_curve3(const gp_Ax2& A,double u,double r,double M,double m,double F,int n,const gp_Pnt& P){
 gp_Pnt p;gp_Vec v1,v2,v3;
 gp_Ax1 L(A.Location(),A.Direction());
 el_pnt(ElCLib::LineValue(u,L));ElCLib::LineD1(u,L,p,v1);el_pnt(p);el_vec(v1);el_vec(ElCLib::LineDN(u,L,n));el_real(ElCLib::LineParameter(L,P));
 el_pnt(ElCLib::CircleValue(u,A,r));ElCLib::CircleD1(u,A,r,p,v1);el_pnt(p);el_vec(v1);
 ElCLib::CircleD2(u,A,r,p,v1,v2);el_pnt(p);el_vec(v1);el_vec(v2);ElCLib::CircleD3(u,A,r,p,v1,v2,v3);el_pnt(p);el_vec(v1);el_vec(v2);el_vec(v3);
 el_vec(ElCLib::CircleDN(u,A,r,n));el_real(ElCLib::CircleParameter(A,P));
 el_pnt(ElCLib::EllipseValue(u,A,M,m));ElCLib::EllipseD1(u,A,M,m,p,v1);el_pnt(p);el_vec(v1);
 ElCLib::EllipseD2(u,A,M,m,p,v1,v2);el_pnt(p);el_vec(v1);el_vec(v2);ElCLib::EllipseD3(u,A,M,m,p,v1,v2,v3);el_pnt(p);el_vec(v1);el_vec(v2);el_vec(v3);
 el_vec(ElCLib::EllipseDN(u,A,M,m,n));EL_TRY(el_real(ElCLib::EllipseParameter(A,M,m,P)))
 el_pnt(ElCLib::HyperbolaValue(u,A,M,m));ElCLib::HyperbolaD1(u,A,M,m,p,v1);el_pnt(p);el_vec(v1);
 ElCLib::HyperbolaD2(u,A,M,m,p,v1,v2);el_pnt(p);el_vec(v1);el_vec(v2);ElCLib::HyperbolaD3(u,A,M,m,p,v1,v2,v3);el_pnt(p);el_vec(v1);el_vec(v2);el_vec(v3);
 el_vec(ElCLib::HyperbolaDN(u,A,M,m,n));el_real(ElCLib::HyperbolaParameter(A,M,m,P));
 el_pnt(ElCLib::ParabolaValue(u,A,F));ElCLib::ParabolaD1(u,A,F,p,v1);el_pnt(p);el_vec(v1);
 ElCLib::ParabolaD2(u,A,F,p,v1,v2);el_pnt(p);el_vec(v1);el_vec(v2);el_vec(ElCLib::ParabolaDN(u,A,F,n));el_real(ElCLib::ParabolaParameter(A,P));
}
void el_overloads3(const gp_Ax2& A,double u,double r,double M,double m,double F,int n,const gp_Pnt& P){
 gp_Pnt p;gp_Vec v1,v2,v3;
 gp_Lin lin(A.Location(),A.Direction());el_pnt(ElCLib::Value(u,lin));el_vec(ElCLib::DN(u,lin,n));el_real(ElCLib::Parameter(lin,P));
 try{gp_Circ c(A,r);el_pnt(ElCLib::Value(u,c));ElCLib::D3(u,c,p,v1,v2,v3);el_vec(v3);el_real(ElCLib::Parameter(c,P));}catch(const Standard_Failure& e){el_fail(e);}
 try{gp_Elips c(A,M,m);el_pnt(ElCLib::Value(u,c));ElCLib::D2(u,c,p,v1,v2);el_vec(v2);el_real(ElCLib::Parameter(c,P));}catch(const Standard_Failure& e){el_fail(e);}
 try{gp_Hypr c(A,M,m);el_pnt(ElCLib::Value(u,c));ElCLib::D1(u,c,p,v1);el_vec(v1);el_real(ElCLib::Parameter(c,P));}catch(const Standard_Failure& e){el_fail(e);}
 try{gp_Parab c(A,F);el_pnt(ElCLib::Value(u,c));ElCLib::D2(u,c,p,v1,v2);el_vec(v2);el_vec(ElCLib::DN(u,c,n));el_real(ElCLib::Parameter(c,P));}catch(const Standard_Failure& e){el_fail(e);}
}
void elclib3_case(std::istream& in){
 gp_Pnt O=el_read_pnt(in);gp_XYZ N=el_read_xyz(in),X=el_read_xyz(in);double u,r,M,m,F;int n;in>>u>>r>>M>>m>>F>>n;gp_Pnt P=el_read_pnt(in);
 try{gp_Ax2 A(O,gp_Dir(N),gp_Dir(X));el_ax2(A);el_curve3(A,u,r,M,m,F,n,P);el_overloads3(A,u,r,M,m,F,n,P);}catch(const Standard_Failure& e){el_fail(e);}
 std::cout<<'\n';
}
void el_curve2(const gp_Ax22d& A,const gp_Ax2d& L,double u,double r,double M,double m,double F,int n,const gp_Pnt2d& P){
 gp_Pnt2d p;gp_Vec2d v1,v2,v3;
 el_pnt2(ElCLib::LineValue(u,L));ElCLib::LineD1(u,L,p,v1);el_pnt2(p);el_vec2(v1);el_vec2(ElCLib::LineDN(u,L,n));el_real(ElCLib::LineParameter(L,P));
 el_pnt2(ElCLib::CircleValue(u,A,r));ElCLib::CircleD1(u,A,r,p,v1);el_pnt2(p);el_vec2(v1);
 ElCLib::CircleD2(u,A,r,p,v1,v2);el_pnt2(p);el_vec2(v1);el_vec2(v2);ElCLib::CircleD3(u,A,r,p,v1,v2,v3);el_pnt2(p);el_vec2(v1);el_vec2(v2);el_vec2(v3);
 el_vec2(ElCLib::CircleDN(u,A,r,n));EL_TRY(el_real(ElCLib::CircleParameter(A,P)))
 el_pnt2(ElCLib::EllipseValue(u,A,M,m));ElCLib::EllipseD1(u,A,M,m,p,v1);el_pnt2(p);el_vec2(v1);
 ElCLib::EllipseD2(u,A,M,m,p,v1,v2);el_pnt2(p);el_vec2(v1);el_vec2(v2);ElCLib::EllipseD3(u,A,M,m,p,v1,v2,v3);el_pnt2(p);el_vec2(v1);el_vec2(v2);el_vec2(v3);
 el_vec2(ElCLib::EllipseDN(u,A,M,m,n));EL_TRY(el_real(ElCLib::EllipseParameter(A,M,m,P)))
 el_pnt2(ElCLib::HyperbolaValue(u,A,M,m));ElCLib::HyperbolaD1(u,A,M,m,p,v1);el_pnt2(p);el_vec2(v1);
 ElCLib::HyperbolaD2(u,A,M,m,p,v1,v2);el_pnt2(p);el_vec2(v1);el_vec2(v2);ElCLib::HyperbolaD3(u,A,M,m,p,v1,v2,v3);el_pnt2(p);el_vec2(v1);el_vec2(v2);el_vec2(v3);
 el_vec2(ElCLib::HyperbolaDN(u,A,M,m,n));el_real(ElCLib::HyperbolaParameter(A,M,m,P));
 el_pnt2(ElCLib::ParabolaValue(u,A,F));ElCLib::ParabolaD1(u,A,F,p,v1);el_pnt2(p);el_vec2(v1);
 ElCLib::ParabolaD2(u,A,F,p,v1,v2);el_pnt2(p);el_vec2(v1);el_vec2(v2);el_vec2(ElCLib::ParabolaDN(u,A,F,n));el_real(ElCLib::ParabolaParameter(A,P));
}
void el_overloads2(const gp_Ax22d& A,const gp_Ax2d& L,double u,double r,double M,double m,double F,int n,const gp_Pnt2d& P){
 gp_Pnt2d p;gp_Vec2d v1,v2,v3;
 gp_Lin2d lin(L);el_pnt2(ElCLib::Value(u,lin));el_vec2(ElCLib::DN(u,lin,n));el_real(ElCLib::Parameter(lin,P));
 try{gp_Circ2d c(A,r);el_pnt2(ElCLib::Value(u,c));ElCLib::D3(u,c,p,v1,v2,v3);el_vec2(v3);el_real(ElCLib::Parameter(c,P));}catch(const Standard_Failure& e){el_fail(e);}
 try{gp_Elips2d c(A,M,m);el_pnt2(ElCLib::Value(u,c));ElCLib::D2(u,c,p,v1,v2);el_vec2(v2);el_real(ElCLib::Parameter(c,P));}catch(const Standard_Failure& e){el_fail(e);}
 try{gp_Hypr2d c(A,M,m);el_pnt2(ElCLib::Value(u,c));ElCLib::D1(u,c,p,v1);el_vec2(v1);el_real(ElCLib::Parameter(c,P));}catch(const Standard_Failure& e){el_fail(e);}
 try{gp_Parab2d c(A,F);el_pnt2(ElCLib::Value(u,c));ElCLib::D2(u,c,p,v1,v2);el_vec2(v2);el_vec2(ElCLib::DN(u,c,n));el_real(ElCLib::Parameter(c,P));}catch(const Standard_Failure& e){el_fail(e);}
}
void elclib2_case(std::istream& in){
 gp_Pnt2d O=el_read_pnt2(in);gp_XY X=el_read_xy(in);int sense;double u,r,M,m,F;int n;in>>sense>>u>>r>>M>>m>>F>>n;gp_Pnt2d P=el_read_pnt2(in);
 try{gp_Dir2d D(X);gp_Ax22d A(O,D,sense!=0);gp_Ax2d L(O,D);el_ax22d(A);el_curve2(A,L,u,r,M,m,F,n,P);el_overloads2(A,L,u,r,M,m,F,n,P);}catch(const Standard_Failure& e){el_fail(e);}
 std::cout<<'\n';
}
void elperiod_case(std::istream& in){
 double u,first,last,precision,u1,u2;in>>u>>first>>last>>precision>>u1>>u2;
 el_real(ElCLib::InPeriod(u,first,last));ElCLib::AdjustPeriodic(first,last,precision,u1,u2);el_real(u1);el_real(u2);std::cout<<'\n';
}
void el_embed(const gp_Ax2& A,const gp_Ax22d& F,const gp_Ax2d& L,double r,double M,double m,double f,const gp_Pnt2d& P,const gp_Vec2d& V){
 el_pnt(ElCLib::To3d(A,P));el_vec(ElCLib::To3d(A,V));EL_TRY(el_dir(ElCLib::To3d(A,L.Direction())))
 EL_TRY(el_ax1(ElCLib::To3d(A,L)))EL_TRY(el_ax2(ElCLib::To3d(A,F)))EL_TRY(el_lin(ElCLib::To3d(A,gp_Lin2d(L))))
 try{el_circ(ElCLib::To3d(A,gp_Circ2d(F,r)));}catch(const Standard_Failure& e){el_fail(e);}
 try{gp_Elips e3=ElCLib::To3d(A,gp_Elips2d(F,M,m));el_ax2(e3.Position());el_real(e3.MajorRadius());el_real(e3.MinorRadius());}catch(const Standard_Failure& e){el_fail(e);}
 try{gp_Hypr h3=ElCLib::To3d(A,gp_Hypr2d(F,M,m));el_ax2(h3.Position());el_real(h3.MajorRadius());el_real(h3.MinorRadius());}catch(const Standard_Failure& e){el_fail(e);}
 try{gp_Parab p3=ElCLib::To3d(A,gp_Parab2d(F,f));el_ax2(p3.Position());el_real(p3.Focal());}catch(const Standard_Failure& e){el_fail(e);}
}
void elto3d_case(std::istream& in){
 gp_Pnt O=el_read_pnt(in);gp_XYZ N=el_read_xyz(in),X=el_read_xyz(in);gp_Pnt2d O2=el_read_pnt2(in);gp_XY X2=el_read_xy(in);int sense;double r,M,m,f;in>>sense>>r>>M>>m>>f;
 gp_Pnt2d P=el_read_pnt2(in);gp_XY V=el_read_xy(in);
 try{gp_Ax2 A(O,gp_Dir(N),gp_Dir(X));gp_Dir2d D(X2);el_embed(A,gp_Ax22d(O2,D,sense!=0),gp_Ax2d(O2,D),r,M,m,f,P,gp_Vec2d(V));}catch(const Standard_Failure& e){el_fail(e);}
 std::cout<<'\n';
}
