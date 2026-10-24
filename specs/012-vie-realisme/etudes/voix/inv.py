import struct,sys
p=sys.argv[1]
d=open(p,'rb').read()
def chunks(b,off,end):
    while off<end:
        t=b[off:off+4]; sz=struct.unpack('<I',b[off+4:off+8])[0]
        yield t,off+8,sz
        off+=8+sz+(sz&1)
ch={}
def walk(off,end):
    for t,o,s in chunks(d,off,end):
        if t==b'LIST':
            lt=d[o:o+4]
            walk(o+4,o+s) if lt in(b'pdta',) else None
            if lt==b'INFO':
                for t2,o2,s2 in chunks(d,o+4,o+s):
                    print('INFO',t2.decode(),d[o2:o2+s2].split(b'\0')[0][:200])
        else: ch[t]=(o,s)
walk(12,len(d))
o,s=ch[b'phdr']
ph=[struct.unpack('<20sHHHIII',d[o+i*38:o+i*38+38]) for i in range(s//38)]
for n,pr,bk,bag,*_ in ph[:-1]:
    n=n.split(b'\0')[0].decode('latin1')
    if len(sys.argv)>2 and (pr in (52,53,54,55,56,57,85,91,94) ) and bk==0 or len(sys.argv)>2 and bk==0 and any(k in n.lower() for k in ('voice','choir','aah','ooh','vox','vocal','vowel')):
        print(bk,pr,n)
print('presets',len(ph)-1)
if len(sys.argv)>2:
    print('--- all banks')
    for n,pr,bk,*_ in ph[:-1]:
        if pr in (52,53,54,85,91): print(bk,pr,n.split(b'\0')[0].decode())
    # full ICMT
    for t2,o2,s2 in chunks(d,12,len(d)):
        pass
