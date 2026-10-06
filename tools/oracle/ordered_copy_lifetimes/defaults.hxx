struct DefaultTrace {occ::handle<NCollection_BaseAllocator> arena;std::string events;bool reject=false;int baseline=0;bool relative=false;
 int Count()const{return arena->GetRefCount()-(relative?baseline:0);}};
static DefaultTrace* defaultTrace=nullptr;
struct DefaultPolicy {
 DefaultPolicy() {
  defaultTrace->events+="HFr"+std::to_string(defaultTrace->Count())+",";
  if(defaultTrace->reject)throw std::runtime_error("default");
 }
 ~DefaultPolicy(){defaultTrace->events+="HDr"+std::to_string(defaultTrace->Count())+",";}
 size_t operator()(int key)const{return key;}bool operator()(int a,int b)const{return a==b;}
};
template<class Map>void defaultBody(int mode) {
 if(mode==0){Map map;std::cout<<"ok "<<map.Size()<<' '<<map.NbBuckets()<<' ';return;}
 if(mode==4){Map map(0);std::cout<<"ok "<<map.Size()<<' '<<map.NbBuckets()<<' ';return;}
 if(mode==1){Map map(size_t(13),defaultTrace->arena);std::cout<<"ok "<<map.Size()<<' '<<map.NbBuckets()<<' ';return;}
 Map map(mode==3?-7:0,defaultTrace->arena);std::cout<<"ok "<<map.Size()<<' '<<map.NbBuckets()<<' ';
}
template<class Map>void defaults(const char* prefix,const char* label,int mode,bool reject=false) {
 DefaultTrace trace;trace.arena=(mode==0||mode==4)?NCollection_BaseAllocator::CommonBaseAllocator():new CopyArena;
 trace.relative=(mode==0||mode==4);trace.baseline=trace.arena->GetRefCount();
 if(trace.relative)std::cerr<<prefix<<'_'<<label<<"_ambient "<<trace.baseline<<'\n';
 trace.reject=reject;defaultTrace=&trace;std::cout<<prefix<<'_'<<label<<' ';
 try{defaultBody<Map>(mode);}catch(const Standard_OutOfRange&){std::cout<<"range ";}
 catch(const std::runtime_error&){std::cout<<"throw ";}
 std::cout<<trace.Count()<<' '<<(trace.events.empty()?"-":trace.events)<<'\n';defaultTrace=nullptr;
}
template<class Map>void defaultSuite(const char* prefix) {
 defaults<Map>(prefix,"default_bare",0);defaults<Map>(prefix,"default_size",1);
 defaults<Map>(prefix,"default_signed",2);defaults<Map>(prefix,"default_negative",3);
 defaults<Map>(prefix,"default_throw_size",1,true);defaults<Map>(prefix,"default_negative_throw",3,true);
 defaults<Map>(prefix,"default_common_int",4);defaults<Map>(prefix,"default_throw_bare",0,true);
 defaults<Map>(prefix,"default_throw_signed",2,true);
}
