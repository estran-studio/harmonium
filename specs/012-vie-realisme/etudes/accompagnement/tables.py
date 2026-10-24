import json,sys
d=json.load(open(sys.argv[1]))
short={"reel_quebecois":"RQ","gigue_quebecoise":"GQ","valse_quebecoise":"VQ","reel_acadien":"RA","gigue_britannique_coloniale":"GB","hornpipe_britannique_coloniale":"HB","marche_britannique_coloniale":"MB"}
order=list(short)
def P(c,k):
    t=sum(c.values()) or 1
    return f"{100*c.get(k,0)/t:.1f}".replace('.',',')
def row(name,cells): print("| "+name+" | "+" | ".join(cells)+" |")
def table(title,head,keys,key,labels=None):
    print(f"\n**{title}**\n"); print("| Profil | n | "+" | ".join(head)+" |"); print("|---|---:|"+"---:|"*len(head))
    for n in order:
        c=d[n]['stats'].get(key,{}); t=sum(c.values())
        row(f"`{n}`",[str(t)]+[P(c,k) for k in keys])
print("annot:")
print("| Profil | airs du type (métrique du profil) | airs avec accords chiffrés | symboles | `/` basse écrite (%) | septièmes (%) |"); print("|---|---:|---:|---:|---:|---:|")
for n in order:
    v=d[n]; row(f"`{n}`",[str(v['tunes_total']),str(v['tunes_annotated']),str(v['symbols']),f"{100*v['slash_symbols']/max(1,v['symbols']):.1f}".replace('.',','),f"{100*v['seventh_symbols']/max(1,v['symbols']):.1f}".replace('.',',')])
table("Changements d'accord effectifs par mesure (%, mesures pleines)",["0 (tenu)","1","2","3+"],["0","1","2","3+"],"changes_per_bar")
table("Position des changements (% des changements, hors premier accord)",["début de mesure","demi-mesure","autre temps","contretemps"],["temps 1 (début de mesure)","demi-mesure","autre temps","contretemps"],"change_position")
table("Distance diatonique entre fondamentales successives (% des changements ; d = degrés montants mod 7)",["d=1 seconde ↑","d=6 seconde ↓","d=2 tierce ↑","d=5 tierce ↓","d=3 quarte ↑","d=4 quinte ↑","d=0 même racine","chromatique"],["1","6","2","5","3","4","0","chromatique"],"root_motion_diatonic")
table("Notes de passage diatoniques possibles dans le plus court sens (% des changements)",["0 (seconde)","1 (tierce)","2 (quarte/quinte)"],["0","1","2"],"passing_available")
print("\n**Passage : position × disponibilité (% des changements diatoniques)**\n")
print("| Profil | frontière de mesure, ≥1 passage | intra-mesure, ≥1 passage | frontière, 0 passage | intra, 0 passage |"); print("|---|---:|---:|---:|---:|")
for n in order:
    c=d[n]['stats']['passing_by_position']; t=sum(c.values())
    f=lambda ks: f"{100*sum(c.get(k,0) for k in ks)/t:.1f}".replace('.',',')
    row(f"`{n}`",[f(['bar_boundary:1','bar_boundary:2']),f(['intra:1','intra:2']),f(['bar_boundary:0']),f(['intra:0'])])
table("Mélodie sur le temps : note de la triade de l'accord actif (%)", ["temps 1","demi-mesure","autre temps"],[],"melody_on_beat") if False else None
print("\n**Mélodie sur le temps : % de notes hors triade de l'accord écrit**\n")
print("| Profil | n notes sur le temps | temps 1 | demi-mesure (ou temps 2 du 6/8) | autres temps |"); print("|---|---:|---:|---:|---:|")
for n in order:
    c=d[n]['stats']['melody_on_beat']; t=sum(c.values())
    def r(k):
        a=c.get(k+':nct',0); b=c.get(k+':ct',0)
        return "—" if a+b==0 else f"{100*a/(a+b):.1f}".replace('.',',')
    row(f"`{n}`",[str(t),r('temps 1'),r('demi-mesure'),r('autre temps')])
print("\n**Fondamentales : décalage en demi-tons montants (% des changements)**\n")
ks=["5","7","2","10","9","3","8","4","1","0","6","11"]
print("| Profil | "+" | ".join(ks)+" |"); print("|---|"+"---:|"*len(ks))
for n in order:
    c=d[n]['stats']['root_motion_semitones_up']; row(f"`{n}`",[P(c,k) for k in ks])
for n in ["reel_quebecois","gigue_quebecoise","valse_quebecoise","gigue_britannique_coloniale","hornpipe_britannique_coloniale"]:
    print(f"\n**Par position de phrase — `{n}` (mesures 1..8 des parties de 8 mesures complètes)**\n")
    print("| Mesure | n | entrée : changement (%) | changement intra-mesure (%) | accord final : I | V | IV | vi | autre |"); print("|---:|---:|---:|---:|---:|---:|---:|---:|---:|")
    for j,p in d[n]['by_position'].items():
        nb=p['n']['bars']; f=p['final']; t=sum(f.values())
        g=lambda k: f"{100*f.get(k,0)/t:.0f}"
        other=max(0,100-sum(round(100*f.get(k,0)/t) for k in ("I","V","IV","vi")))
        row(j,[str(nb),f"{100*p['entry']['change']/nb:.0f}",f"{100*p['intra']['change']/nb:.0f}",g("I"),g("V"),g("IV"),g("vi"),str(other)])
