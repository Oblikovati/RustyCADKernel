// Full dictionary and sentence oracle; byte strings retain OCCT's legacy encoding.
#include <UnitsAPI.hxx>
#include <Units_UnitSentence.hxx>
#include <Units_Token.hxx>
#include <Units_Dimensions.hxx>
#include <Standard_OutOfRange.hxx>
#include <Standard_Failure.hxx>
#include <iostream>
#include <sstream>
#include <cstring>
#include <cstdint>
std::string unhex(const std::string& text){std::string value;for(size_t i=0;i<text.size();i+=2)value.push_back(static_cast<char>(std::stoi(text.substr(i,2),nullptr,16)));return value;}
std::string hex(const char* text){const char* digits="0123456789abcdef";std::string value;for(const unsigned char* c=(const unsigned char*)text;*c;++c){value+=digits[*c>>4];value+=digits[*c&15];}return value;}
uint64_t bits(double value){uint64_t result;std::memcpy(&result,&value,8);return result;}
void run(std::istream& input){
  std::string encoded,target;double value;input>>encoded>>value>>target;
  std::string expression=unhex(encoded),destination=unhex(target);
  UnitsAPI::SetLocalSystem(UnitsAPI_SI);
  Units_UnitSentence sentence(expression.c_str());
  std::ostringstream out;out<<sentence.IsDone();
  if(sentence.IsDone()){
    auto token=sentence.Evaluate();if(token.IsNull()){std::cout<<"null\n";return;}
    out<<' '<<hex(token->Word().ToCString())<<' '<<bits(token->Value());auto dims=token->Dimensions();
    for(double d:{dims->Mass(),dims->Length(),dims->Time(),dims->ElectricCurrent(),dims->ThermodynamicTemperature(),dims->AmountOfSubstance(),dims->LuminousIntensity(),dims->PlaneAngle(),dims->SolidAngle()})out<<' '<<bits(d);
  }
  auto to=UnitsAPI::AnyToSI(value,expression.c_str());auto from=UnitsAPI::AnyFromSI(value,expression.c_str());
  // Unknown-unit diagnostics go to native printers; suppress only the known warning sink via stdout buffering.
  std::ostringstream warnings;auto original=std::cout.rdbuf(warnings.rdbuf());
  double converted;try{converted=UnitsAPI::AnyToAny(value,expression.c_str(),destination.c_str());}catch(...){std::cout.rdbuf(original);throw;}
  std::cout.rdbuf(original);UnitsAPI::SetLocalSystem(UnitsAPI_MDTV);auto local=UnitsAPI::AnyToLS(value,expression.c_str());
  out<<' '<<bits(to)<<' '<<bits(from)<<' '<<bits(converted)<<' '<<bits(local);std::cout<<out.str()<<'\n';
}
int main(){std::string line;while(std::getline(std::cin,line)){std::istringstream input(line);try{run(input);}catch(Standard_OutOfRange const& e){std::cout<<"error Standard_OutOfRange "<<hex(e.what())<<'\n';}catch(Standard_Failure const& e){std::cout<<"error Standard_Failure "<<hex(e.what())<<'\n';}}}
