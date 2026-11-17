"""Measure WCAG 2.x contrast for Aegis tokens. Usage: python contrast.py ../aegis.css"""
import re,sys
css=open(sys.argv[1],encoding='utf8').read()
def block(sel):
    m=re.search(re.escape(sel)+r'\{(.*?)\}',css,re.S);return {k:v.strip() for k,v in re.findall(r'--([\w-]+):([^;]+);',m.group(1))}
def rgba(s):
    s=s.strip()
    if s.startswith('#'):
        h=s[1:];return tuple(int(h[i:i+2],16) for i in (0,2,4))+(1.0,)
    m=re.match(r'rgba?\(([^)]+)\)',s);p=[float(x) for x in m.group(1).split(',')];return (p[0],p[1],p[2],p[3] if len(p)>3 else 1.0)
def over(fg,bg):
    a=fg[3];return tuple(fg[i]*a+bg[i]*(1-a) for i in range(3))+(1.0,)
def lum(c):
    f=lambda v:(v/255/12.92) if v/255<=.03928 else ((v/255+.055)/1.055)**2.4
    return .2126*f(c[0])+.7152*f(c[1])+.0722*f(c[2])
def cr(a,b):
    l1,l2=sorted([lum(a),lum(b)],reverse=True);return (l1+.05)/(l2+.05)
rows=[];fails=0
for name,sel in [('dark',':root,[data-theme="dark"]'),('light','[data-theme="light"]')]:
    t={k:rgba(v) for k,v in block(sel).items() if re.match(r'(#|rgba?)',v)}
    bg,card,bg2,elev=t['bg'],t['card'],t['bg2'],t['elev']
    def tint(k,base):return over(t[k],base)
    tests=[('text','bg',t['text'],bg,4.5),('text','card',t['text'],card,4.5),('text2','bg',t['text2'],bg,4.5),('text2','card',t['text2'],card,4.5),('text2','bg2',t['text2'],bg2,4.5),
    ('text3','elev',t['text3'],elev,4.5),('text2','elev',t['text2'],elev,4.5),('text','elev',t['text'],elev,4.5),('crit-text','crit-soft/elev',t['crit-text'],over(t['crit-soft'],elev),4.5),('crit-text','crit-soft/accent-soft/card',t['crit-text'],over(t['crit-soft'],over(t['accent-soft'],card)),4.5),('text3','accent-soft/card',t['text3'],over(t['accent-soft'],card),4.5),
    ('text3','card',t['text3'],card,4.5),('text3','bg',t['text3'],bg,4.5),('text3','bg2',t['text3'],bg2,4.5),
    ('accent-text','card',t['accent-text'],card,4.5),('accent-text','accent-soft/card',t['accent-text'],tint('accent-soft',card),4.5),
    ('accent-ink on accent (primary button)',' ',t['accent-ink'],over(t['accent'],card),4.5),
    ('warn','card',t['warn'],card,4.5),('warn','warn-soft/card',t['warn'],tint('warn-soft',card),4.5),
    ('crit','card',t['crit'],card,4.5),('crit','crit-soft/card',t['crit'],tint('crit-soft',card),4.5),
    ('ok','card',t['ok'],card,4.5),('ok','ok-soft/card',t['ok'],tint('ok-soft',card),4.5),
    ('info','card',t['info'],card,4.5),('info','info-soft/card',t['info'],tint('info-soft',card),4.5),
    ('text','add/card (diff ins)',t['text'],tint('add',card),4.5),('text','del/card (diff del)',t['text'],tint('del',card),4.5),
    ('focus ring accent','bg (non-text 3:1)',t['accent'],bg,3.0),('focus ring accent','card (3:1)',t['accent'],card,3.0),
    ('edge (control border)','bg2 (3:1)',t['edge'],bg2,3.0),('edge','card (3:1)',t['edge'],card,3.0)]
    for a,b,fg,bgc,need in tests:
        r=cr(fg,bgc);ok=r>=need;fails+=not ok;rows.append((name,a,b,round(r,2),need,'PASS' if ok else 'FAIL'))
print('| theme | foreground | background | ratio | needs | result |\n|---|---|---|---|---|---|')
for r in rows:print('| '+' | '.join(map(str,r))+' |')
print('\nFAILS:',fails)
