// OCCT 3d097a0: Clear borrows its handle through every node destructor before allocator assignment.
#include "ordered_borrowed_clear/fixture.hxx"
#include "ordered_borrowed_clear/termination.hxx"
template<class Map>void normalModes(const char* family,bool common) {
 for(int mode=0;mode<4;++mode)for(int state=0;state<3;++state)normalCase<Map>(family,common,mode,state);
}
template<class Map>void clearSuite(const char* family) {
 for(bool common:{false,true})normalModes<Map>(family,common);
 for(int mode=0;mode<2;++mode)normalCase<Map>(family,false,mode,1,true);
 for(int mode=0;mode<4;++mode)terminationCase<Map>(family,mode,'K');
}
int main(){clearSuite<ClearSet>("set");clearSuite<ClearMap>("data");for(int mode=0;mode<4;++mode)terminationCase<ClearMap>("data",mode,'V');}
