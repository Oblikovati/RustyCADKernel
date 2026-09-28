// TKG2d oracle: points, directions, vectors, axis placements, transformations and the
// Geom2d_Geometry transformation members (tkg2d_probe.cxx).
// The Mirror/Rotate/Scale/Translate members and their copying *ed twins for one geometry.
void g2_moves(const occ::handle<Geom2d_Geometry>& g,G2Reader& r,void (*show)(const occ::handle<Geom2d_Geometry>&)){
 gp_Pnt2d p=r.pnt();gp_Ax2d a=r.ax();double angle=r.real(),factor=r.real();gp_Vec2d v=r.vec();gp_Pnt2d q=r.pnt();
 gp_Trsf2d t=g2_trsf(r);
 G2_TRY(show(g->Mirrored(p)));G2_TRY(show(g->Mirrored(a)));G2_TRY(show(g->Rotated(p,angle)));
 G2_TRY(show(g->Scaled(p,factor)));G2_TRY(show(g->Transformed(t)));G2_TRY(show(g->Translated(v)));
 G2_TRY(show(g->Translated(p,q)));
 G2_TRY(g->Mirror(p);show(g));G2_TRY(g->Mirror(a);show(g));G2_TRY(g->Rotate(p,angle);show(g));
 G2_TRY(g->Scale(p,factor);show(g));G2_TRY(g->Translate(v);show(g));G2_TRY(g->Translate(p,q);show(g));
 G2_TRY(g->Transform(t);show(g));
}
void g2_show_point(const occ::handle<Geom2d_Geometry>& g){
 occ::handle<Geom2d_CartesianPoint> p=occ::down_cast<Geom2d_CartesianPoint>(g);double x,y;p->Coord(x,y);
 g2_real(x);g2_real(y);g2_pnt(p->Pnt2d());g2_real(p->X());g2_real(p->Y());g2_dump(p,-1);
}
void g2_point_case(std::istream& in){
 G2Reader r{in};gp_Pnt2d a=r.pnt();gp_Pnt2d b=r.pnt();double x=r.real(),y=r.real();
 occ::handle<Geom2d_CartesianPoint> p=new Geom2d_CartesianPoint(a.X(),a.Y());
 occ::handle<Geom2d_CartesianPoint> o=new Geom2d_CartesianPoint(b);
 g2_real(p->Distance(o));g2_real(p->SquareDistance(o));g2_show_point(p->Copy());
 p->SetCoord(x,y);g2_show_point(p);p->SetX(y);p->SetY(x);g2_show_point(p);p->SetPnt2d(b);g2_show_point(p);
 g2_dump(p,0);g2_dump(p,1);g2_moves(p,r,g2_show_point);std::cout<<'\n';
}
void g2_show_vector(const occ::handle<Geom2d_Geometry>& g){
 occ::handle<Geom2d_Vector> v=occ::down_cast<Geom2d_Vector>(g);double x,y;v->Coord(x,y);
 g2_real(x);g2_real(y);g2_vec(v->Vec2d());g2_real(v->X());g2_real(v->Y());g2_real(v->Magnitude());
 g2_real(v->SquareMagnitude());g2_dump(v,-1);
}
void g2_vector_pair(const occ::handle<Geom2d_Vector>& v,const occ::handle<Geom2d_Vector>& o){
 G2_TRY(g2_real(v->Angle(o)));g2_real(v->Dot(o));g2_real(v->Crossed(o));G2_TRY(g2_show_vector(v->Reversed()));
}
void g2_direction_case(std::istream& in){
 G2Reader r{in};double x=r.real(),y=r.real(),a=r.real(),b=r.real();gp_Vec2d other=r.vec();
 occ::handle<Geom2d_Direction> d;
 try{d=new Geom2d_Direction(x,y);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 occ::handle<Geom2d_VectorWithMagnitude> o=new Geom2d_VectorWithMagnitude(other);
 g2_show_vector(d);g2_dir(d->Dir2d());g2_vector_pair(d,o);g2_real(d->operator^(o));
 G2_TRY(d->SetCoord(a,b));g2_show_vector(d);G2_TRY(d->SetX(a));g2_show_vector(d);G2_TRY(d->SetY(b));g2_show_vector(d);
 d->SetDir2d(gp_Dir2d(1,1));g2_show_vector(d);d->Reverse();g2_show_vector(d);g2_show_vector(d->Copy());
 g2_moves(d,r,g2_show_vector);std::cout<<'\n';
}
void g2_vector_case(std::istream& in){
 G2Reader r{in};gp_Vec2d a=r.vec(),b=r.vec();double s=r.real();gp_Pnt2d p=r.pnt(),q=r.pnt();
 occ::handle<Geom2d_VectorWithMagnitude> v=new Geom2d_VectorWithMagnitude(a.X(),a.Y());
 occ::handle<Geom2d_VectorWithMagnitude> o=new Geom2d_VectorWithMagnitude(b);
 g2_show_vector(new Geom2d_VectorWithMagnitude(p,q));g2_vector_pair(v,o);
 g2_show_vector(v->Added(o));g2_show_vector(v->Subtracted(o));g2_show_vector(v->Multiplied(s));
 G2_TRY(g2_show_vector(v->Divided(s)));G2_TRY(g2_show_vector(v->Normalized()));
 g2_show_vector(v->operator+(o));g2_show_vector(v->operator-(o));g2_show_vector(v->operator/(s));g2_real(v->operator^(o));
 v->Add(o);g2_show_vector(v);v->Subtract(o);g2_show_vector(v);v->Multiply(s);g2_show_vector(v);
 G2_TRY(v->Divide(s));g2_show_vector(v);G2_TRY(v->Normalize());g2_show_vector(v);
 v->operator+=(o);v->operator-=(o);v->operator*=(s);v->operator/=(s);g2_show_vector(v);
 v->SetCoord(s,-s);g2_show_vector(v);v->SetX(a.X());v->SetY(a.Y());g2_show_vector(v);v->SetVec2d(b);g2_show_vector(v);
 g2_moves(v,r,g2_show_vector);std::cout<<'\n';
}
void g2_show_axis(const occ::handle<Geom2d_Geometry>& g){
 occ::handle<Geom2d_AxisPlacement> a=occ::down_cast<Geom2d_AxisPlacement>(g);
 g2_ax(a->Ax2d());g2_dir(a->Direction());g2_pnt(a->Location());g2_dump(a,-1);
}
void g2_axis_case(std::istream& in){
 G2Reader r{in};gp_Ax2d first=r.ax();gp_Pnt2d p=r.pnt();gp_Dir2d d=r.dir();gp_Ax2d other=r.ax();
 occ::handle<Geom2d_AxisPlacement> a=new Geom2d_AxisPlacement(first);
 occ::handle<Geom2d_AxisPlacement> o=new Geom2d_AxisPlacement(p,d);
 g2_real(a->Angle(o));g2_show_axis(a->Reversed());a->Reverse();g2_show_axis(a);a->SetLocation(p);g2_show_axis(a);
 a->SetDirection(d);g2_show_axis(a);a->SetAxis(other);g2_show_axis(a);g2_show_axis(a->Copy());
 g2_moves(a,r,g2_show_axis);std::cout<<'\n';
}
void g2_trsf_state(const occ::handle<Geom2d_Transformation>& t){
 g2_flag(t->IsNegative());g2_int(t->Form());g2_real(t->ScaleFactor());
 for(int row=0;row<=3;++row)for(int col=0;col<=4;++col)G2_TRY(g2_real(t->Value(row,col)));
 double x=1.25,y=-3.5;t->Transforms(x,y);g2_real(x);g2_real(y);
}
void g2_trsf_case(std::istream& in){
 G2Reader r{in};int kind=r.integer();gp_Pnt2d p=r.pnt(),q=r.pnt();gp_Ax2d a=r.ax(),b=r.ax();double s=r.real();int n=r.integer();
 occ::handle<Geom2d_Transformation> t=new Geom2d_Transformation();
 if(kind==0)t->SetMirror(p);
 if(kind==1)t->SetMirror(a);
 if(kind==2)t->SetRotation(p,s);
 if(kind==3)t->SetScale(p,s);
 if(kind==4)t->SetTransformation(a,b);
 if(kind==5)t->SetTransformation(a);
 if(kind==6)t->SetTranslation(gp_Vec2d(p,q));
 if(kind==7)t->SetTranslation(p,q);
 if(kind==8){gp_Trsf2d g;g.SetRotation(q,s);t->SetTrsf2d(g);}
 g2_trsf_state(t);occ::handle<Geom2d_Transformation> o=new Geom2d_Transformation(t->Trsf2d());o->SetRotation(q,0.75);
 G2_TRY(g2_trsf_state(t->Inverted()));g2_trsf_state(t->Multiplied(o));g2_trsf_state(t->operator*(o));
 G2_TRY(g2_trsf_state(t->Powered(n)));g2_trsf_state(t->Copy());
 occ::handle<Geom2d_Transformation> c=t->Copy();c->Multiply(o);g2_trsf_state(c);c=t->Copy();c->operator*=(o);g2_trsf_state(c);
 c=t->Copy();c->PreMultiply(o);g2_trsf_state(c);c=t->Copy();G2_TRY(c->Power(n));g2_trsf_state(c);
 c=t->Copy();G2_TRY(c->Invert());g2_trsf_state(c);std::cout<<'\n';
}
