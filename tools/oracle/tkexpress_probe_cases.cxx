// The direct API scenarios of the TKExpress oracle (`ex_case <n>`), the Rust twin of
// crates/tkexpress/examples/ex_probe/cases.rs: members the interpreter never reaches,
// and every native exception the Expr classes raise.
static UnknownH ex_x(const char* name){return new Expr_NamedUnknown(TCollection_AsciiString(name));}
static ExprH ex_v(double value){return new Expr_NumericValue(value);}

// 0: SetOperand checks of the unary, binary and poly expressions.
static void ex_case_operands(){
 UnknownH x=ex_x("x");occ::handle<Expr_Sine> s=new Expr_Sine(x);occ::handle<Expr_UnaryMinus> m=new Expr_UnaryMinus(s);
 ex_try([&]{s->SetOperand(s);});ex_try([&]{s->SetOperand(m);});ex_try([&]{s->SetOperand(ex_v(2.));ex_text(m->String());});
 occ::handle<Expr_Difference> d=new Expr_Difference(x,ex_v(1.));occ::handle<Expr_Sum> p=new Expr_Sum(d,x);
 ex_try([&]{d->SetFirstOperand(p);});ex_try([&]{d->SetSecondOperand(d);});ex_try([&]{d->SetSecondOperand(ex_v(3.));ex_text(p->String());});
 ex_try([&]{p->SetOperand(ex_v(4.),3);});ex_try([&]{p->SetOperand(p,1);});ex_try([&]{p->SetOperand(ex_v(5.),2);ex_text(p->String());});
 ex_try([&]{ex_text(p->SubExpression(3)->String());});ex_try([&]{ex_text(d->SubExpression(0)->String());});
 ex_try([&]{ex_text(s->SubExpression(2)->String());});}
// 1: named unknowns: assignment, its checks and the members that follow it.
static void ex_case_unknowns(){
 UnknownH x=ex_x("x");UnknownH y=ex_x("y");
 ex_try([&]{ex_text(x->AssignedExpression()->String());});ex_try([&]{ex_text(x->SubExpression(1)->String());});
 ex_try([&]{x->Assign(new Expr_Sum(x,ex_v(1.)));});
 ex_try([&]{x->Assign(new Expr_Sum(y,ex_v(1.)));ex_bool(x->IsAssigned());ex_int(x->NbSubExpressions());});
 ex_try([&]{ex_text(x->String());ex_text(x->Simplified()->String());ex_text(x->ShallowSimplified()->String());});
 ex_try([&]{ex_bool(x->ContainsUnknowns());ex_bool(x->Contains(y));ex_bool(x->IsLinear());});
 ex_try([&]{ex_text(x->Derivative(y)->String());ex_text(x->Derivative(x)->String());});
 ex_try([&]{ExprH c=x->Copy();ex_text(c->String());ex_bool(c->IsIdentical(x));ex_bool(c==x);});
 ex_try([&]{NCollection_Array1<UnknownH> vr(1,1);NCollection_Array1<double> vl(1,1);vr(1)=y;vl(1)=2.5;ex_real(x->Evaluate(vr,vl));});
 ex_try([&]{x->Replace(y,ex_v(7.));ex_text(x->AssignedExpression()->String());});
 ex_try([&]{x->Deassign();ex_bool(x->IsAssigned());ex_int(x->NbSubExpressions());ex_bool(x->ContainsUnknowns());});
 ex_try([&]{NCollection_Array1<UnknownH> vr(1,1);NCollection_Array1<double> vl(1,1);vr(1)=y;vl(1)=2.5;ex_real(x->Evaluate(vr,vl));});
 ex_try([&]{NCollection_Array1<UnknownH> vr(0,1);NCollection_Array1<double> vl(5,6);vr(0)=y;vr(1)=x;vl(5)=1.;vl(6)=9.;ex_real(x->Evaluate(vr,vl));});
 ex_try([&]{occ::handle<Expr_NamedConstant> k=new Expr_NamedConstant("K",2.);ex_text(k->String());ex_bool(k->IsIdentical(x));
  ex_text(k->NDerivative(x,0)->String());ex_text(k->Simplified()->String());});}
// 2: the N <= 0 checks of NDerivative and the derivative members of the operators.
static void ex_case_derivatives(){
 UnknownH x=ex_x("x");ExprH s=new Expr_Sum(new Expr_Sine(x),x);
 ex_try([&]{ex_text(s->NDerivative(x,0)->String());});ex_try([&]{ex_text(ex_v(2.)->NDerivative(x,-1)->String());});
 ex_try([&]{ex_text(ExprH(new Expr_Sine(x))->NDerivative(x,0)->String());});
 ex_try([&]{ex_text(ExprH(new Expr_UnaryMinus(x))->NDerivative(x,0)->String());});
 ex_try([&]{ex_text(ExprH(new Expr_Difference(x,ex_v(1.)))->NDerivative(x,0)->String());});
 ex_try([&]{ex_text(ExprH(new Expr_Difference(new Expr_Sine(x),x))->NDerivative(x,3)->String());});
 ex_try([&]{ex_text(ExprH(new Expr_UnaryMinus(new Expr_Cosine(x)))->NDerivative(x,2)->String());});
 ex_try([&]{ex_text(ExprH(new Expr_Absolute(ex_v(3.)))->Derivative(x)->String());});
 ex_try([&]{ex_text(ExprH(new Expr_Sign(x))->Derivative(x)->String());});}
// 3: named functions, their derivatives and the function expressions.
static void ex_case_functions(){
 UnknownH x=ex_x("x");UnknownH y=ex_x("y");NCollection_Array1<UnknownH> v1(1,1);v1(1)=x;NCollection_Array1<UnknownH> v2(1,2);v2(1)=x;v2(2)=y;
 occ::handle<Expr_NamedFunction> f=new Expr_NamedFunction("f",new Expr_Product(x,new Expr_Sine(x)),v1);
 occ::handle<Expr_NamedFunction> g=new Expr_NamedFunction("g",new Expr_Sum(new Expr_Product(x,y),new Expr_Square(y)),v2);
 ex_try([&]{occ::handle<Expr_FunctionDerivative> d=new Expr_FunctionDerivative(f,x,0);});
 ex_try([&]{occ::handle<Expr_UnaryFunction> u=new Expr_UnaryFunction(g,x);});
 ex_try([&]{occ::handle<Expr_BinaryFunction> b=new Expr_BinaryFunction(f,x,y);});
 ex_try([&]{NCollection_Array1<UnknownH> vr(1,2);NCollection_Array1<double> vl(1,1);vr(1)=x;vr(2)=y;vl(1)=1.;ex_real(f->Evaluate(vr,vl));});
 ex_try([&]{occ::handle<Expr_GeneralFunction> d=f->Derivative(x,2);ex_text(d->GetStringName());ex_int(d->NbOfVariables());
  ex_text(occ::down_cast<Expr_FunctionDerivative>(d)->Expression()->String());ex_bool(d->IsLinearOnVariable(1));
  occ::handle<Expr_GeneralFunction> dd=d->Derivative(x);ex_text(dd->GetStringName());ex_bool(d->IsIdentical(f->Derivative(x,2)));
  ex_bool(d->IsIdentical(dd));ex_text(d->Copy()->GetStringName());});
 ex_try([&]{occ::handle<Expr_GeneralFunction> d=g->Derivative(y,2);ex_text(d->GetStringName());
  occ::handle<Expr_GeneralFunction> dx=d->Derivative(x);ex_text(dx->GetStringName());
  ex_text(occ::down_cast<Expr_FunctionDerivative>(dx)->Expression()->String());
  NCollection_Array1<UnknownH> vr(1,2);NCollection_Array1<double> vl(1,2);vr(1)=x;vr(2)=y;vl(1)=0.75;vl(2)=-1.5;ex_real(dx->Evaluate(vr,vl));});
 ex_try([&]{ExprH u=new Expr_UnaryFunction(f,new Expr_Square(y));ex_text(u->String());ex_text(u->Derivative(y)->String());
  ex_bool(u->IsLinear());ex_text(u->Copy()->String());ex_bool(u->IsIdentical(u->Copy()));
  ex_text(ExprH(new Expr_UnaryFunction(f,ex_v(0.25)))->ShallowSimplified()->String());});
 ex_try([&]{ExprH b=new Expr_BinaryFunction(g,new Expr_Sine(y),ex_v(2.));ex_text(b->String());ex_text(b->Derivative(y)->String());
  ex_bool(b->IsLinear());ex_text(ExprH(new Expr_BinaryFunction(g,ex_v(1.5),ex_v(2.)))->ShallowSimplified()->String());});
 ex_try([&]{NCollection_Array1<ExprH> a(1,2);a(1)=y;a(2)=new Expr_Cosine(y);ExprH p=new Expr_PolyFunction(g,a);ex_text(p->String());
  ex_text(p->Derivative(y)->String());ex_bool(p->IsLinear());ex_bool(p->IsIdentical(p->Copy()));
  NCollection_Array1<ExprH> c(1,2);c(1)=ex_v(0.5);c(2)=ex_v(2.);ex_text(ExprH(new Expr_PolyFunction(g,c))->ShallowSimplified()->String());});
 ex_try([&]{f->SetName("h");ex_text(f->GetStringName());f->SetExpression(new Expr_Cosine(x));ex_text(f->Expression()->String());
  ex_bool(f->IsIdentical(f->Copy()));ex_bool(f->IsIdentical(g));ex_text(f->Variable(1)->GetName());});
 ex_try([&]{ex_text(f->Variable(2)->GetName());});}
// 4: replacement inside expressions and relations.
static void ex_case_replace(){
 UnknownH x=ex_x("x");UnknownH y=ex_x("y");ExprH e=new Expr_Product(new Expr_Sine(x),new Expr_Sum(x,y));
 ex_try([&]{e->Replace(x,new Expr_Sum(y,ex_v(1.)));ex_text(e->String());});
 ex_try([&]{occ::handle<Expr_Sine> s=new Expr_Sine(y);s->Replace(y,new Expr_UnaryMinus(s));});
 ex_try([&]{ExprH q=new Expr_Division(y,new Expr_Exponentiate(y,ex_v(2.)));q->Replace(y,ex_v(3.));ex_text(q->String());
  ex_text(q->ShallowSimplified()->String());ex_text(q->Simplified()->String());});
 ex_try([&]{UnknownH z=ex_x("z");z->Assign(new Expr_Sum(x,ex_v(1.)));z->Replace(x,y);ex_text(z->AssignedExpression()->String());});}
// 5: the interpreter outside a successful parse.
static void ex_case_interpreter(){
 occ::handle<ExprIntrp_GenExp> g=ExprIntrp_GenExp::Create();
 ex_try([&]{ex_bool(g->IsDone());ex_text(g->Expression()->String());});
 ex_try([&]{g->Process("");ex_bool(g->IsDone());});
 ex_try([&]{g->Process("x+y");ex_bool(g->IsDone());ex_text(g->Expression()->String());
  ex_bool(g->GetNamed("x").IsNull());ex_bool(g->GetNamed("q").IsNull());ex_bool(g->GetFunction("x").IsNull());});
 ex_try([&]{g->Process("x+");ex_bool(g->IsDone());ex_text(g->Expression()->String());});}
// 6: the iterators past their end.
static void ex_case_iterators(){
 UnknownH x=ex_x("x");ExprH e=new Expr_Sum(x,new Expr_Sine(x));
 ex_try([&]{Expr_UnknownIterator it(e);ex_bool(it.More());it.Next();ex_bool(it.More());it.Next();});
 ex_try([&]{Expr_UnknownIterator it(e);it.Next();ex_text(it.Value()->GetName());});
 ex_try([&]{occ::handle<Expr_Equal> q=new Expr_Equal(x,ex_v(1.));Expr_RelationIterator it(q);it.Next();ex_text(it.Value()->String());});
 ex_try([&]{occ::handle<Expr_Equal> q=new Expr_Equal(x,ex_v(1.));Expr_RelationIterator it(q);it.Next();it.Next();});
 ex_try([&]{occ::handle<Expr_Equal> q=new Expr_Equal(x,ex_v(1.));Expr_RUIterator it(q);it.Next();ex_text(it.Value()->GetName());});
 ex_try([&]{occ::handle<Expr_Equal> q=new Expr_Equal(x,ex_v(1.));ex_text(q->SubRelation(1)->String());});}
// 7: the operators with reals keep the native operand order.
static void ex_case_operators(){
 UnknownH x=ex_x("x");ExprH e=x;
 ex_try([&]{ex_text((e+2.)->String());ex_text((2.+e)->String());ex_text((e-2.)->String());ex_text((2.-e)->String());});
 ex_try([&]{ex_text((e*2.)->String());ex_text((2.*e)->String());ex_text((e/2.)->String());ex_text((2./e)->String());});
 ex_try([&]{ex_text((-e)->String());ex_text((e+e)->String());ex_text((e-e)->String());ex_text((e*e)->String());ex_text((e/e)->String());});}
// 8: IsIdentical of sums and products, operand order and repeated operands.
static void ex_case_identical(){
 UnknownH x=ex_x("x");UnknownH y=ex_x("y");
 NCollection_Sequence<ExprH> a;a.Append(x);a.Append(x);a.Append(y);NCollection_Sequence<ExprH> b;b.Append(x);b.Append(y);b.Append(x);
 NCollection_Sequence<ExprH> c;c.Append(x);c.Append(y);c.Append(y);
 ex_try([&]{ExprH p=new Expr_Product(a),q=new Expr_Product(b),r=new Expr_Product(c);
  ex_bool(p->IsIdentical(q));ex_bool(q->IsIdentical(p));ex_bool(p->IsIdentical(r));ex_bool(r->IsIdentical(p));});
 ex_try([&]{ExprH p=new Expr_Sum(a),q=new Expr_Sum(b),r=new Expr_Sum(c);
  ex_bool(p->IsIdentical(q));ex_bool(p->IsIdentical(r));ex_bool(p->IsIdentical(new Expr_Product(a)));});
 ex_try([&]{occ::handle<Expr_Sum> p=new Expr_Sum(a);p->SetOperand(ex_v(1.),1);ex_text(p->String());ex_text(p->ShallowSimplified()->String());});
 ex_try([&]{ExprH d=new Expr_Difference(x,y);ex_bool(d->IsIdentical(new Expr_Difference(x,y)));ex_bool(d->IsIdentical(new Expr_Difference(y,x)));
  ex_bool(ExprH(new Expr_Exponentiate(x,y))->IsIdentical(new Expr_Exponentiate(x,y)));ex_bool(ExprH(new Expr_Division(x,y))->IsIdentical(new Expr_Division(y,x)));});}

// `ex_case <n>`: one scenario.
static void ex_case_case(std::istream& in){int n;in>>n;
 static void(*const scenarios[])()={ex_case_operands,ex_case_unknowns,ex_case_derivatives,ex_case_functions,
  ex_case_replace,ex_case_interpreter,ex_case_iterators,ex_case_operators,ex_case_identical};
 if(n>=0&&n<9)scenarios[n]();ex_end();}
