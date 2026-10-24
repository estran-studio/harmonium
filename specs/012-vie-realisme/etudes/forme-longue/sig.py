import json, collections
d=json.load(open('/tmp/set_etude/sets.json'))
by=collections.defaultdict(list)
for r in d: by[r['tuneset']].append((int(r['settingorder']),r['tune_id'],r['member_id']))
sets=[]
for sid,m in by.items():
    m.sort(); 
    if len(m)>=2: sets.append((m[0][2],[x[1] for x in m]))
# retours d'un air chez un même membre
per=collections.defaultdict(lambda: collections.defaultdict(int))
for mem,tl in sets:
    for t in set(tl): per[mem][t]+=1
tot=ret=0; nmem=0; mem_sets=collections.Counter()
for mem,dd in per.items():
    tot+=len(dd); ret+=sum(1 for v in dd.values() if v>=2)
for mem,tl in sets: mem_sets[mem]+=1
print('membres',len(per),'sets>=2 airs',len(sets))
print('airs distincts (membre,air)',tot,'dans >=2 sets du même membre',ret,f'{100*ret/tot:.1f}%')
big=[m for m,c in mem_sets.items() if c>=10]
tot2=ret2=0
for m in big:
    dd=per[m]; tot2+=len(dd); ret2+=sum(1 for v in dd.values() if v>=2)
print('membres >=10 sets:',len(big),'airs',tot2,'retours',ret2,f'{100*ret2/tot2:.1f}%')
# part des membres-sets où l'air apparaît dans un autre set du même membre
inst=0; instret=0
for mem,tl in sets:
    for t in tl:
        inst+=1
        if per[mem][t]>=2: instret+=1
print('airs-instances',inst,'dont air présent dans >=2 sets du membre',f'{100*instret/inst:.1f}%')
# récurrence des paires adjacentes entre membres différents
pm=collections.defaultdict(set); pc=collections.Counter()
for mem,tl in sets:
    for a,b in zip(tl,tl[1:]):
        pm[(a,b)].add(mem); pc[(a,b)]+=1
npairs=sum(pc.values())
rec=sum(c for p,c in pc.items() if len(pm[p])>=2)
print('transitions',npairs,'paires distinctes',len(pc),f'transitions dont la paire (a→b) apparaît chez ≥2 membres : {100*rec/npairs:.1f}%')
# concentration : part des airs-instances couverts par les 100 airs les plus fréquents
tc=collections.Counter(t for _,tl in sets for t in tl)
top=sum(c for _,c in tc.most_common(100)); allc=sum(tc.values())
print('top100 airs couvrent',f'{100*top/allc:.1f}%','airs distincts',len(tc))
