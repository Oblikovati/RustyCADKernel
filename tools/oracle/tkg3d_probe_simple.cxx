// TKG3d oracle: points, directions, vectors, axis placements, transformations and the
// Geom_Geometry transformation members (tkg3d_probe.cxx).
// The Mirror/Rotate/Scale/Translate members and their copying *ed twins for one geometry.
// Inputs: P, axis (P D), frame (P N X), angle, factor, V, Q, transformation.
void g3_moves(const occ::handle<Geom_Geometry>& g,G3Reader& r,void (*show)(const occ::handle<Geom_Geometry>&)){
 gp_Pnt p=r.pnt();gp_Ax1 a=r.ax1();gp_Ax2 f=r.ax2();double angle=r.real(),factor=r.real();gp_Vec v=r.vec();gp_Pnt q=r.pnt();
 gp_Trsf t=g3_trsf(r);
 G3_TRY(show(g->Mirrored(p)));G3_TRY(show(g->Mirrored(a)));G3_TRY(show(g->Mirrored(f)));G3_TRY(show(g->Rotated(a,angle)));
 G3_TRY(show(g->Scaled(p,factor)));G3_TRY(show(g->Transformed(t)));G3_TRY(show(g->Translated(v)));
 G3_TRY(show(g->Translated(p,q)));
 G3_TRY(g->Mirror(p);show(g));G3_TRY(g->Mirror(a);show(g));G3_TRY(g->Mirror(f);show(g));G3_TRY(g->Rotate(a,angle);show(g));
 G3_TRY(g->Scale(p,factor);show(g));G3_TRY(g->Translate(v);show(g));G3_TRY(g->Translate(p,q);show(g));
 G3_TRY(g->Transform(t);show(g));
}
void g3_show_point(const occ::handle<Geom_Geometry>& g){
 occ::handle<Geom_CartesianPoint> p=occ::down_cast<Geom_CartesianPoint>(g);double x,y,z;p->Coord(x,y,z);
 g3_real(x);g3_real(y);g3_real(z);g3_pnt(p->Pnt());g3_real(p->X());g3_real(p->Y());g3_real(p->Z());g3_dump(p,-1);
}
void g3_point_case(std::istream& in){
 G3Reader r{in};gp_Pnt a=r.pnt();gp_Pnt b=r.pnt();double x=r.real(),y=r.real(),z=r.real();
 occ::handle<Geom_CartesianPoint> p=new Geom_CartesianPoint(a.X(),a.Y(),a.Z());
 occ::handle<Geom_CartesianPoint> o=new Geom_CartesianPoint(b);
 g3_real(p->Distance(o));g3_real(p->SquareDistance(o));g3_show_point(p->Copy());
 p->SetCoord(x,y,z);g3_show_point(p);p->SetX(y);p->SetY(z);p->SetZ(x);g3_show_point(p);p->SetPnt(b);g3_show_point(p);
 g3_dump(p,0);g3_dump(p,1);g3_moves(p,r,g3_show_point);std::cout<<'\n';
}
void g3_show_vector(const occ::handle<Geom_Geometry>& g){
 occ::handle<Geom_Vector> v=occ::down_cast<Geom_Vector>(g);double x,y,z;v->Coord(x,y,z);
 g3_real(x);g3_real(y);g3_real(z);g3_vec(v->Vec());g3_real(v->X());g3_real(v->Y());g3_real(v->Z());
 g3_real(v->Magnitude());g3_real(v->SquareMagnitude());g3_dump(v,-1);
}
// The members shared by all vectors, against `o` and the reference `ref`.
void g3_vector_pair(const occ::handle<Geom_Vector>& v,const occ::handle<Geom_Vector>& o,const occ::handle<Geom_Vector>& ref){
 G3_TRY(g3_real(v->Angle(o)));G3_TRY(g3_real(v->AngleWithRef(o,ref)));g3_real(v->Dot(o));g3_real(v->DotCross(o,ref));
 G3_TRY(g3_show_vector(v->Crossed(o)));G3_TRY(g3_show_vector(v->CrossCrossed(o,ref)));G3_TRY(g3_show_vector(v->Reversed()));
 G3_TRY(g3_show_vector(v->Copy()));
}
// The in-place cross products on copies, so the next steps see the original vector.
void g3_vector_cross(const occ::handle<Geom_Vector>& v,const occ::handle<Geom_Vector>& o,const occ::handle<Geom_Vector>& ref){
 occ::handle<Geom_Vector> c=occ::down_cast<Geom_Vector>(v->Copy());G3_TRY(c->Cross(o));g3_show_vector(c);
 c=occ::down_cast<Geom_Vector>(v->Copy());G3_TRY(c->CrossCross(o,ref));g3_show_vector(c);
 c=occ::down_cast<Geom_Vector>(v->Copy());G3_TRY(c->Cross(c));g3_show_vector(c);
}
void g3_direction_case(std::istream& in){
 G3Reader r{in};double x=r.real(),y=r.real(),z=r.real(),a=r.real(),b=r.real(),c=r.real();gp_Vec other=r.vec(),ref=r.vec();
 occ::handle<Geom_Direction> d;
 try{d=new Geom_Direction(x,y,z);}catch(const Standard_Failure& e){std::cout<<e.ExceptionType()<<'\n';return;}
 occ::handle<Geom_VectorWithMagnitude> o=new Geom_VectorWithMagnitude(other),f=new Geom_VectorWithMagnitude(ref);
 g3_show_vector(d);g3_dir(d->Dir());g3_vector_pair(d,o,f);g3_vector_cross(d,o,f);
 G3_TRY(d->SetCoord(a,b,c));g3_show_vector(d);G3_TRY(d->SetX(a));g3_show_vector(d);G3_TRY(d->SetY(b));g3_show_vector(d);
 G3_TRY(d->SetZ(c));g3_show_vector(d);d->SetDir(gp_Dir(1,1,1));g3_show_vector(d);d->Reverse();g3_show_vector(d);
 g3_moves(d,r,g3_show_vector);std::cout<<'\n';
}
void g3_vector_case(std::istream& in){
 G3Reader r{in};gp_Vec a=r.vec(),b=r.vec(),ref=r.vec();double s=r.real();gp_Pnt p=r.pnt(),q=r.pnt();
 occ::handle<Geom_VectorWithMagnitude> v=new Geom_VectorWithMagnitude(a.X(),a.Y(),a.Z());
 occ::handle<Geom_VectorWithMagnitude> o=new Geom_VectorWithMagnitude(b),f=new Geom_VectorWithMagnitude(ref);
 g3_show_vector(new Geom_VectorWithMagnitude(p,q));g3_vector_pair(v,o,f);g3_vector_cross(v,o,f);
 g3_show_vector(v->Added(o));g3_show_vector(v->Subtracted(o));g3_show_vector(v->Multiplied(s));
 G3_TRY(g3_show_vector(v->Divided(s)));G3_TRY(g3_show_vector(v->Normalized()));
 v->Add(o);g3_show_vector(v);v->Subtract(o);g3_show_vector(v);v->Multiply(s);g3_show_vector(v);
 G3_TRY(v->Divide(s));g3_show_vector(v);G3_TRY(v->Normalize());g3_show_vector(v);
 v->SetCoord(s,-s,2*s);g3_show_vector(v);v->SetX(a.X());v->SetY(a.Y());v->SetZ(a.Z());g3_show_vector(v);
 v->SetVec(b);g3_show_vector(v);g3_moves(v,r,g3_show_vector);std::cout<<'\n';
}
void g3_show_axis1(const occ::handle<Geom_Geometry>& g){
 occ::handle<Geom_Axis1Placement> a=occ::down_cast<Geom_Axis1Placement>(g);
 g3_ax1(a->Ax1());g3_ax1(a->Axis());g3_dir(a->Direction());g3_pnt(a->Location());g3_dump(a,-1);
}
void g3_axis1_case(std::istream& in){
 G3Reader r{in};gp_Ax1 a=r.ax1();gp_Ax1 b=r.ax1();gp_Dir d=r.dir();gp_Pnt p=r.pnt();
 occ::handle<Geom_Axis1Placement> x=new Geom_Axis1Placement(a);
 occ::handle<Geom_Axis1Placement> y=new Geom_Axis1Placement(b.Location(),b.Direction());
 g3_show_axis1(x);g3_real(x->Angle(y));g3_show_axis1(x->Reversed());g3_show_axis1(x->Copy());
 x->Reverse();g3_show_axis1(x);x->SetDirection(d);g3_show_axis1(x);x->SetLocation(p);g3_show_axis1(x);
 x->SetAxis(b);g3_show_axis1(x);g3_moves(x,r,g3_show_axis1);std::cout<<'\n';
}
void g3_show_axis2(const occ::handle<Geom_Geometry>& g){
 occ::handle<Geom_Axis2Placement> a=occ::down_cast<Geom_Axis2Placement>(g);
 G3_TRY(g3_ax2(a->Ax2()));g3_ax1(a->Axis());g3_dir(a->XDirection());g3_dir(a->YDirection());g3_dump(a,-1);
}
void g3_axis2_case(std::istream& in){
 G3Reader r{in};gp_Ax2 f=r.ax2();gp_Pnt p=r.pnt();gp_Dir n=r.dir(),vx=r.dir(),d=r.dir(),e=r.dir();gp_Ax1 a=r.ax1();
 occ::handle<Geom_Axis2Placement> x=new Geom_Axis2Placement(f),y;
 try{y=new Geom_Axis2Placement(p,n,vx);}catch(const Standard_Failure& err){std::cout<<err.ExceptionType()<<' ';y=x;}
 g3_show_axis2(x);g3_show_axis2(y);g3_real(x->Angle(y));g3_show_axis2(y->Copy());
 G3_TRY(x->SetDirection(d));g3_show_axis2(x);G3_TRY(x->SetXDirection(e));g3_show_axis2(x);
 G3_TRY(x->SetYDirection(d));g3_show_axis2(x);x->SetLocation(p);g3_show_axis2(x);x->SetAxis(a);g3_show_axis2(x);
 x->SetAx2(f);g3_show_axis2(x);g3_moves(x,r,g3_show_axis2);std::cout<<'\n';
}
void g3_show_trsf(const occ::handle<Geom_Transformation>& t){
 g3_flag(t->IsNegative());g3_int(t->Form());g3_real(t->ScaleFactor());
 for(int i=1;i<=3;++i)for(int j=1;j<=4;++j)g3_real(t->Value(i,j));G3_TRY(g3_real(t->Value(0,1)));
 double x=1.5,y=-2.,z=0.25;t->Transforms(x,y,z);g3_real(x);g3_real(y);g3_real(z);
 std::ostringstream s;t->DumpJson(s,-1);std::cout<<'{'<<s.str()<<"} ";
}
void g3_trsf_case(std::istream& in){
 G3Reader r{in};gp_Trsf a=g3_trsf(r),b=g3_trsf(r);int n=r.integer();gp_Ax3 f=r.ax3(),h=r.ax3();gp_Pnt p=r.pnt(),q=r.pnt();
 gp_Ax1 ax=r.ax1();gp_Ax2 pl=r.ax2();double angle=r.real(),factor=r.real();gp_Vec v=r.vec();
 occ::handle<Geom_Transformation> t=new Geom_Transformation(a),o=new Geom_Transformation(b),e=new Geom_Transformation();
 g3_show_trsf(e);g3_show_trsf(t);g3_show_trsf(t->Copy());g3_show_trsf(t->Multiplied(o));G3_TRY(g3_show_trsf(t->Inverted()));
 G3_TRY(g3_show_trsf(t->Powered(n)));
 occ::handle<Geom_Transformation> c=t->Copy();c->Multiply(o);g3_show_trsf(c);c->PreMultiply(o);g3_show_trsf(c);
 G3_TRY(c->Power(n));g3_show_trsf(c);G3_TRY(c->Invert());g3_show_trsf(c);
 c->SetMirror(p);g3_show_trsf(c);c->SetMirror(ax);g3_show_trsf(c);c->SetMirror(pl);g3_show_trsf(c);
 G3_TRY(c->SetRotation(ax,angle));g3_show_trsf(c);G3_TRY(c->SetScale(p,factor));g3_show_trsf(c);
 c->SetTransformation(f,h);g3_show_trsf(c);c->SetTransformation(f);g3_show_trsf(c);
 c->SetTranslation(v);g3_show_trsf(c);c->SetTranslation(p,q);g3_show_trsf(c);c->SetTrsf(a);g3_show_trsf(c);
 std::ostringstream s;t->DumpJson(s,0);std::cout<<'{'<<s.str()<<"} "<<'\n';
}
