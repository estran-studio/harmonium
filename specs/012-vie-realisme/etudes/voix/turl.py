import mido, numpy as np, soundfile as sf, librosa, sys
OUT='/Users/william.quintal/Project/bascanada/harmonium_lab/target/voix_etude/'
SR=44100
def load(name):
    y,sr=sf.read(f'/tmp/v8voix/syl/{name}.wav'); y=y if y.ndim==1 else y.mean(1)
    env=np.abs(y); idx=np.where(env>0.03*env.max())[0]; y=y[idx[0]:idx[-1]]
    f0,vf,_=librosa.pyin(y.astype(np.float32),fmin=70,fmax=500,sr=sr)
    f=np.nanmedian(f0) if np.isfinite(f0).any() else 180.0
    return y,float(f)
S={n:load(n) for n in ['ta','ti','di','li','lam','lon','la']}
print({k:(round(len(v[0])/SR,3),round(v[1],1)) for k,v in S.items()})
def notes(mid,ch=0):
    m=mido.MidiFile(mid); t=0;on={};out=[]
    for x in m.tracks[0]:
        t+=x.time
        if getattr(x,'channel',None)!=ch: continue
        sec=mido.tick2second(t,m.ticks_per_beat,500000)  # tempo fixed below
        if x.type=='note_on' and x.velocity>0: on[x.note]=(t,x.velocity)
        elif x.type in('note_off','note_on') and x.note in on:
            t0,v=on.pop(x.note); out.append((t0,t,x.note,v))
    return m,out
def render(style,shift=-12,outname=None):
    m,ns=notes(OUT+style+'.mid')
    tempo=[x.tempo for x in m.tracks[0] if x.type=='set_tempo'][0]
    tps=lambda t: mido.tick2second(t,m.ticks_per_beat,tempo)
    total=tps(max(n[1] for n in ns))+1.0
    y=np.zeros(int(total*SR)); tpb=m.ticks_per_beat
    prev=None
    for t0,t1,n,v in ns:
        beat=(t0%tpb)/tpb; dur=tps(t1-t0)
        if beat==0: syl='lam' if dur>=0.9*tps(tpb) else 'ta'
        elif beat==0.5: syl='di' if prev!=n else 'ti'
        else: syl='li'
        prev=n
        smp,f0=S[syl]
        ft=440*2**((n+shift-69)/12); ratio=ft/f0
        L=int(len(smp)/ratio); z=np.interp(np.linspace(0,len(smp)-1,L),np.arange(len(smp)),smp)
        maxL=int((max(dur,0.09)+0.03)*SR) if syl!='lam' else int(max(dur,0.2)*SR)
        z=z[:maxL]; fo=min(len(z),int(0.02*SR)); z[-fo:]*=np.linspace(1,0,fo)
        i=int(tps(t0)*SR); y[i:i+len(z)]+=z*(0.5+0.5*v/127)
    y/=np.abs(y).max()*1.05
    sf.write(OUT+f'{style}_turlutte_proto_syllabes_say_voix_seule.wav',y,SR)
    # mix with GM full render
    g,gs=sf.read(OUT+f'{style}_gm_complet.wav'); g=g.mean(1) if g.ndim>1 else g
    n=max(len(g),len(y)); mix=np.zeros(n); mix[:len(g)]+=g/np.abs(g).max()*0.5; mix[:len(y)]+=y*0.5
    sf.write(OUT+f'{style}_turlutte_proto_syllabes_say_avec_violon.wav',mix/np.abs(mix).max()*0.9,SR)
    print(style,len(ns),'notes', round(len(y)/SR,1),'s')
for s in ['reel_quebecois','gigue_quebecoise']: render(s)
