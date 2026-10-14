import ctypes as c
import json
import sys
from pathlib import Path
x=c.CDLL('libX11.so.6')
x.XOpenDisplay.argtypes=[c.c_char_p];x.XOpenDisplay.restype=c.c_void_p
x.XGetImage.argtypes=[c.c_void_p,c.c_ulong,c.c_int,c.c_int,c.c_uint,c.c_uint,c.c_ulong,c.c_int];x.XGetImage.restype=c.c_void_p
x.XGetPixel.argtypes=[c.c_void_p,c.c_int,c.c_int];x.XGetPixel.restype=c.c_ulong
x.XDestroyImage.argtypes=[c.c_void_p];x.XCloseDisplay.argtypes=[c.c_void_p]
d=x.XOpenDisplay(b':1');image=x.XGetImage(d,int(sys.argv[1],0),8,8,300,110,c.c_ulong(-1),2)
assert image
rgba=bytearray()
for yy in range(110):
 for xx in range(300):
  pixel=x.XGetPixel(image,xx,yy);rgba.extend([(pixel>>16)&255,(pixel>>8)&255,pixel&255,255])
x.XDestroyImage(image);x.XCloseDisplay(d)
Path(sys.argv[2]).write_bytes(rgba)
expected=Path('/tmp/native-gui-GUI replacements before drawing-expected.rgba').read_bytes()
region=b''.join(expected[4*(y*1440+8):4*(y*1440+308)] for y in range(8,118))
print(json.dumps({'native_gui_mismatch_pixels':sum(rgba[i:i+4]!=region[i:i+4] for i in range(0,len(rgba),4)),'first_pixel':list(rgba[:4]),'path':sys.argv[2]}),flush=True)
