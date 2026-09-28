#include <string>
#include <iostream>
#include <functional>
int main(){for(const char* name:{"One","Two","Three","Four"})std::cout<<name<<" "<<std::hash<std::string>{}(name)<<"\n";}
