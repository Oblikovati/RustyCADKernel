#include <iostream>
#include <string>
#include <functional>
int main(){ for(const char* s:{"A","B","C"})std::cout<<s<<" "<<std::hash<std::string>{}(s)<<"\n"; for(double v:{0.0,-0.0,1.0,2.0,3.0,0.1,1.1,-1.0})std::cout<<v<<" "<<std::hash<double>{}(v)<<"\n"; }
