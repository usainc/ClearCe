"""Developer build step: finalize VERSIONINFO before Tauri Authenticode signing.
Uses Windows resource APIs, not a packer. Never shipped or executed by Setup.
"""
import ctypes as c
from ctypes import wintypes as w
import json
from pathlib import Path
import struct
import sys

ROOT = Path(__file__).resolve().parents[1]

def align(n):
    return (n + 3) & ~3

def parse(data, offset=0):
    length, value_length, kind = struct.unpack_from('<HHH', data, offset)
    end = offset + length
    cursor = offset + 6
    start = cursor
    while data[cursor:cursor+2] != b'\0\0':
        cursor += 2
    key = data[start:cursor].decode('utf-16le')
    cursor = align(cursor + 2)
    size = value_length * 2 if kind else value_length
    value = data[cursor:cursor+size]
    cursor = align(cursor + size)
    children = []
    while cursor + 6 <= end:
        child, cursor = parse(data, cursor)
        children.append(child)
        cursor = align(cursor)
    return [key, kind, value, children], end

def encode(node):
    key, kind, value, children = node
    result = bytearray(b'\0'*6 + (key+'\0').encode('utf-16le'))
    result.extend(b'\0' * (align(len(result))-len(result)))
    result.extend(value)
    for child in children:
        result.extend(b'\0' * (align(len(result))-len(result)))
        result.extend(encode(child))
    struct.pack_into('<HHH', result, 0, len(result), len(value)//2 if kind else len(value), kind)
    return bytes(result)

def finalize(path):
    pe = path.read_bytes()
    for source in (str(ROOT), ROOT.as_posix()):
        if any(source.encode(encoding) in pe for encoding in ('utf-8', 'utf-16le')):
            raise RuntimeError('Developer source path embedded; use npm run release:windows to remap paths.')
    pe_offset = struct.unpack_from('<I', pe, 0x3c)[0]
    optional = pe_offset + 24
    directory = optional + (112 if struct.unpack_from('<H', pe, optional)[0] == 0x20b else 96)
    if any(struct.unpack_from('<II', pe, directory + 4 * 8)):
        raise RuntimeError('Refusing to modify a signed PE. Rebuild an unsigned executable first.')
    k = c.WinDLL('kernel32', use_last_error=True)
    for name, args, result in [
        ('LoadLibraryExW', [w.LPCWSTR,w.HANDLE,w.DWORD],w.HMODULE),
        ('FindResourceW',[w.HMODULE,w.LPCWSTR,w.LPCWSTR],w.HANDLE),
        ('SizeofResource',[w.HMODULE,w.HANDLE],w.DWORD),
        ('LoadResource',[w.HMODULE,w.HANDLE],w.HANDLE),
        ('LockResource',[w.HANDLE],c.c_void_p),
        ('FreeLibrary',[w.HMODULE],w.BOOL),
        ('BeginUpdateResourceW',[w.LPCWSTR,w.BOOL],w.HANDLE),
        ('UpdateResourceW',[w.HANDLE,w.LPCWSTR,w.LPCWSTR,w.WORD,c.c_void_p,w.DWORD],w.BOOL),
        ('EndUpdateResourceW',[w.HANDLE,w.BOOL],w.BOOL),
    ]:
        fn=getattr(k,name);fn.argtypes=args;fn.restype=result
    integer=lambda n:c.cast(c.c_void_p(n),w.LPCWSTR)
    module=k.LoadLibraryExW(str(path),None,2)
    if not module: raise c.WinError(c.get_last_error())
    try:
        resource=k.FindResourceW(module,integer(1),integer(16))
        if not resource: raise c.WinError(c.get_last_error())
        data=c.string_at(k.LockResource(k.LoadResource(module,resource)),k.SizeofResource(module,resource))
    finally: k.FreeLibrary(module)
    tree,_=parse(data)
    for child in tree[3]:
        if child[0]=='StringFileInfo':
            for table in child[3]:
                for name,value in {'FileDescription':'ClearCe — Local AI Image Enhancer','OriginalFilename':'clearce.exe'}.items():
                    table[3]=[item for item in table[3] if item[0]!=name]
                    table[3].append([name,1,(value+'\0').encode('utf-16le'),[]])
    updated=encode(tree)
    handle=k.BeginUpdateResourceW(str(path),False)
    if not handle: raise c.WinError(c.get_last_error())
    if not k.UpdateResourceW(handle,integer(16),integer(1),0x409,c.c_char_p(updated),len(updated)):
        k.EndUpdateResourceW(handle,True)
        raise c.WinError(c.get_last_error())
    if not k.EndUpdateResourceW(handle,False): raise c.WinError(c.get_last_error())
    print('Finalized ClearCe VERSIONINFO (before signing)')

if __name__=='__main__':
    profile='debug' if '--debug' in sys.argv else 'release'
    finalize(ROOT/'src-tauri'/'target'/profile/'clearce.exe')
