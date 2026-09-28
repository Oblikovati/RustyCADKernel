// Uniform-scale and circle behavior, including native compound Power update order.
#include <gp_Circ.hxx>
void transform_values(const gp_Trsf& transform,const gp_Pnt& point) {
  std::cout<<int(transform.Form())<<' '<<bits(transform.ScaleFactor())<<' ';xyz(transform.TranslationPart());matrix(transform.HVectorialPart());xyz(point.Transformed(transform).XYZ());
}
void circle_values(const gp_Circ& circle,const gp_Pnt& point) {
  xyz(circle.Location().XYZ());xyz(circle.Position().Direction().XYZ());xyz(circle.Position().XDirection().XYZ());xyz(circle.Position().YDirection().XYZ());
  std::cout<<bits(circle.Radius())<<' '<<bits(circle.Area())<<' '<<bits(circle.Length())<<' '<<bits(circle.SquareDistance(point))<<' ';
}
void scale_impl(std::istream& input) {
  double v[14];for(auto& value:v)input>>value;
  gp_Pnt point(v[0],v[1],v[2]),origin(v[3],v[4],v[5]);gp_Dir direction(v[6],v[7],v[8]);gp_Ax1 axis(origin,direction);
  gp_Trsf first,second,rotation,translation;first.SetScale(origin,v[10]);second.SetScale(point,v[11]);rotation.SetRotation(axis,v[9]);translation.SetTranslation(gp_Vec(point.XYZ()));
  transform_values(first,point);transform_values(first.Inverted(),point);transform_values(first.Powered(int(v[12])),point);
  for(const gp_Trsf& result:{first*second,first*rotation,rotation*first,translation*first,first*translation,rotation*first*second,first*rotation*second}) {
    transform_values(result,point);transform_values(result.Inverted(),point);transform_values(result.Powered(int(v[12])),point);
  }
  xyz(direction.Transformed(first).XYZ());xyz(direction.Transformed(rotation*first).XYZ());
  gp_Circ circle(gp_Ax2(origin,direction),std::abs(v[13]));circle_values(circle,point);circle_values(circle.Scaled(point,v[10]),point);circle_values(circle.Transformed(first),point);circle_values(circle.Rotated(axis,v[9]),point);circle_values(circle.Mirrored(point),point);std::cout<<'\n';
}

void scale_case(std::istream& input) {
  try {scale_impl(input);} catch(const Standard_Failure& failure) {std::cout<<failure.ExceptionType()<<' '<<failure.GetMessageString()<<'\n';}
}
