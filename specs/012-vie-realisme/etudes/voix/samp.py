import struct,sys
d=open('/Users/william.quintal/Project/bascanada/harmonium/assets/GeneralUser-GS.sf2','rb').read()
ch={}
def walk(off,end):
    while off<end:
        t=d[off:off+4]; sz=struct.unpack('<I',d[off+4:off+8])[0]
        if t==b'LIST':
            lt=d[off+8:off+12]
            if lt==b'pdta': walk(off+12,off+8+sz)
        else: ch[t]=(off+8,sz)
        off+=8+sz+(sz&1)
walk(12,len(d))
def rd(tag,fmt,sz):
    o,s=ch[tag]; return [struct.unpack(fmt,d[o+i*sz:o+i*sz+sz]) for i in range(s//sz)]
ph=rd(b'phdr','<20sHHHIII',38); pb=rd(b'pbag','<HH',4); pg=rd(b'pgen','<Hh',4)
ins=rd(b'inst','<20sH',22); ib=rd(b'ibag','<HH',4); ig=rd(b'igen','<Hh',4)
sh=rd(b'shdr','<20sIIIIIBbHH',46)
def nm(b): return b.split(b'\0')[0].decode('latin1')
def pres(bank,prog):
    for i,(n,p,b,bag,*_) in enumerate(ph[:-1]):
        if p==prog and b==bank:
            nb=ph[i+1][3]
            insts=[]
            for bi in range(bag,nb):
                for g in range(pb[bi][0],pb[bi+1][0]):
                    if pg[g][0]==41: insts.append(pg[g][1])
            print(f'== {bank}/{prog} {nm(n)} instruments',[nm(ins[k][0]) for k in insts])
            for k in insts:
                s0,s1=ins[k][1],ins[k+1][1]
                for bi in range(ib[s0] and s0, s1):
                    gens={ig[g][0]:ig[g][1] for g in range(ib[bi][0],ib[bi+1][0])}
                    if 53 in gens:
                        s=sh[gens[53]]
                        kr=gens.get(43); vr=gens.get(44)
                        kr=(kr&255,kr>>8&255) if kr is not None else None
                        vr=(vr&255,vr>>8&255) if vr is not None else None
                        print(f'   sample {nm(s[0]):20} len={s[2]-s[1]:7} ({(s[2]-s[1])/s[5]:.2f}s) sr={s[5]} root={s[6]} keys={kr} vel={vr} loop={s[4]-s[3]} atk={gens.get(34)} rel={gens.get(38)}')
for a in sys.argv[1:]:
    b,p=map(int,a.split('/')); pres(b,p)
