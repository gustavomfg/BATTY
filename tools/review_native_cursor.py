#!/usr/bin/env python3
"""Linux/X11 rehearsal using real pointer events, including leaving the window.

python tools/review_native_cursor.py target/debug/batpet-desktop /tmp/review-native
Bevy records the desired path; XTest moves the actual pointer, including
departure, which winit's window-relative setter cannot do. No clicks are sent.
"""
import ctypes as c
import os
import math
import re
from pathlib import Path
import subprocess
import sys
import time

x11 = c.CDLL('libX11.so.6')
xtest = c.CDLL('libXtst.so.6')
x11.XOpenDisplay.restype = c.c_void_p
x11.XDefaultRootWindow.argtypes = [c.c_void_p]
x11.XDefaultRootWindow.restype = c.c_ulong
x11.XDisplayHeight.argtypes = [c.c_void_p, c.c_int]
x11.XFlush.argtypes = [c.c_void_p]
x11.XCloseDisplay.argtypes = [c.c_void_p]
x11.XTranslateCoordinates.argtypes = [c.c_void_p,c.c_ulong,c.c_ulong,c.c_int,c.c_int,c.POINTER(c.c_int),c.POINTER(c.c_int),c.POINTER(c.c_ulong)]
x11.XCreateSimpleWindow.argtypes = [c.c_void_p,c.c_ulong,c.c_int,c.c_int,c.c_uint,c.c_uint,c.c_uint,c.c_ulong,c.c_ulong]
x11.XCreateSimpleWindow.restype = c.c_ulong
x11.XMapWindow.argtypes = [c.c_void_p,c.c_ulong]
x11.XDestroyWindow.argtypes = [c.c_void_p,c.c_ulong]
x11.XStoreName.argtypes = [c.c_void_p,c.c_ulong,c.c_char_p]
x11.XQueryPointer.argtypes = [c.c_void_p, c.c_ulong, c.POINTER(c.c_ulong),
    c.POINTER(c.c_ulong), c.POINTER(c.c_int), c.POINTER(c.c_int),
    c.POINTER(c.c_int), c.POINTER(c.c_int), c.POINTER(c.c_uint)]
xtest.XTestFakeMotionEvent.argtypes = [c.c_void_p, c.c_int, c.c_int, c.c_int, c.c_ulong]
display = x11.XOpenDisplay(None)
if not display:
    raise SystemExit('An X11/Xwayland DISPLAY is required')
root, child = c.c_ulong(), c.c_ulong()
old_x, old_y, local_x, local_y, mask = c.c_int(), c.c_int(), c.c_int(), c.c_int(), c.c_uint()
x11.XQueryPointer(display, x11.XDefaultRootWindow(display), c.byref(root),
    c.byref(child), c.byref(old_x), c.byref(old_y), c.byref(local_x),
    c.byref(local_y), c.byref(mask))

def move(x, y):
    xtest.XTestFakeMotionEvent(display, -1, x, y, 0)
    x11.XFlush(display)

directory = Path(sys.argv[2]).resolve()
directory.mkdir(parents=True, exist_ok=True)
environment = dict(os.environ)
environment.pop('WAYLAND_DISPLAY', None)
environment['WINIT_UNIX_BACKEND'] = 'x11'
outside_y = x11.XDisplayHeight(display, 0) - 100
parking = x11.XCreateSimpleWindow(display,x11.XDefaultRootWindow(display),100,outside_y,64,64,0,0,0)
x11.XStoreName(display,parking,b'BATTY cursor review target')
x11.XMapWindow(display,parking)
x11.XFlush(display)

def park():
    # Xwayland may ignore warps into a Wayland-owned surface. A disposable
    # X11 target lets the pointer actually leave BATTY and emit CursorLeft.
    x,y,child = c.c_int(),c.c_int(),c.c_ulong()
    x11.XTranslateCoordinates(display,parking,x11.XDefaultRootWindow(display),0,0,c.byref(x),c.byref(y),c.byref(child))
    move(x.value+32,y.value+32)
process = None
window_id = None
try:
    park()
    with (directory / 'runtime.log').open('w') as log:
        process = subprocess.Popen([str(Path(sys.argv[1]).resolve()), '--debug',
            '--review-dir', str(directory), '--review-presence', '--review-os-cursor', '--review-external-cursor'],
            env=environment, stdout=log, stderr=subprocess.STDOUT)
        started = time.monotonic()
        while process.poll() is None:
            if time.monotonic() - started > 60:
                process.terminate()
                raise RuntimeError('Review exceeded 60 seconds')
            trace = directory / 'cursor.csv'
            rows = trace.read_text().splitlines() if trace.exists() else []
            if window_id is None:
                clients = subprocess.check_output(['xprop','-root','_NET_CLIENT_LIST'],text=True)
                for candidate in re.findall(r'0x[0-9a-fA-F]+',clients):
                    name = subprocess.check_output(['xprop','-id',candidate,'WM_NAME'],text=True)
                    if '"BATTY Desktop"' in name:
                        window_id = int(candidate,16)
                        break
            sample = rows[-1].split(',') if len(rows) > 1 else []
            target_x = float(sample[4]) if len(sample) >= 6 else math.nan
            if math.isnan(target_x) or window_id is None:
                park()
            else:
                origin_x,origin_y,unused = c.c_int(),c.c_int(),c.c_ulong()
                x11.XTranslateCoordinates(display,window_id,x11.XDefaultRootWindow(display),0,0,
                    c.byref(origin_x),c.byref(origin_y),c.byref(unused))
                move(origin_x.value+round(target_x),origin_y.value+round(float(sample[5])))
            time.sleep(0.04)
        if process.returncode:
            raise RuntimeError(f'Renderer exited with {process.returncode}; see runtime.log')
finally:
    if process is not None and process.poll() is None:
        process.terminate()
        process.wait(timeout=5)
    move(old_x.value, old_y.value)
    x11.XDestroyWindow(display,parking)
    x11.XCloseDisplay(display)
print(f'Native cursor rehearsal complete: {directory}')
