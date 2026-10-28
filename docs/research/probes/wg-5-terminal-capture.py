"""Capture the real executable in a disposable tmux server; synthetic data only."""
import json, os, pathlib, re, shlex, subprocess, sys, tempfile, time, unicodedata
from PIL import Image, ImageDraw, ImageFont

executable = str(pathlib.Path(sys.argv[1]).resolve())
out = pathlib.Path(sys.argv[2]); out.mkdir(parents=True, exist_ok=True)
readability = '--readability' in sys.argv[3:]
server = f'wg5-qa-{os.getpid()}'

def tmux(*args):
    return subprocess.check_output(['tmux', '-L', server, *map(str,args)], text=True)

def until(target, needle):
    deadline=time.monotonic()+8
    while time.monotonic()<deadline:
        content=tmux('capture-pane','-p','-t',target)
        if needle in content:return
        time.sleep(.03)
    raise RuntimeError(f'missing {needle}: {content}')

colors=['#000000','#cd0000','#00cd00','#cdcd00','#0000ee','#cd00cd','#00cdcd','#e5e5e5',
        '#7f7f7f','#ff0000','#00ff00','#ffff00','#5c5cff','#ff00ff','#00ffff','#ffffff']
def indexed(n):
    if n<16:return colors[n]
    if n>=232:
        v=8+10*(n-232);return (v,v,v)
    n-=16;r,n=divmod(n,36);g,b=divmod(n,6);lv=[0,95,135,175,215,255]
    return (lv[r],lv[g],lv[b])
fontdir=pathlib.Path('/usr/share/fonts/truetype/dejavu')
font=ImageFont.truetype(str(fontdir/'DejaVuSansMono.ttf'),14)
boldfont=ImageFont.truetype(str(fontdir/'DejaVuSansMono-Bold.ttf'),14)
def png(ansi,width,height,dest):
    image=Image.new('RGB',(width*9+24,height*19+24),'#101010');draw=ImageDraw.Draw(image)
    foreground='#e5e5e5';background='#101010';bold=False;reverse=False
    for row,line in enumerate(ansi.splitlines()[:height]):
        col=0
        for part in re.split(r'(\x1b\[[0-9;]*m)',line):
            if part.startswith('\x1b['):
                codes=[int(s or 0) for s in part[2:-1].split(';')];i=0
                while i<len(codes):
                    c=codes[i];i+=1
                    if c==0:foreground='#e5e5e5';background='#101010';bold=False;reverse=False
                    elif c==1:bold=True
                    elif c==22:bold=False
                    elif c==7:reverse=True
                    elif c==27:reverse=False
                    elif 30<=c<=37:foreground=colors[c-30]
                    elif 90<=c<=97:foreground=colors[c-90+8]
                    elif 40<=c<=47:background=colors[c-40]
                    elif 100<=c<=107:background=colors[c-100+8]
                    elif c==39:foreground='#e5e5e5'
                    elif c==49:background='#101010'
                    elif c in (38,48) and i<len(codes):
                        mode=codes[i];i+=1
                        if mode==5 and i<len(codes):value=indexed(codes[i]);i+=1
                        elif mode==2 and i+2<len(codes):value=tuple(codes[i:i+3]);i+=3
                        else:continue
                        if c==38:foreground=value
                        else:background=value
                continue
            for ch in part:
                if col>=width:break
                fg,bg=(background,foreground) if reverse else (foreground,background)
                x,y=12+col*9,12+row*19
                draw.rectangle((x,y,x+8,y+18),fill=bg)
                draw.text((x,y),ch,font=boldfont if bold else font,fill=fg)
                col+=0 if unicodedata.combining(ch) else 2 if unicodedata.east_asian_width(ch) in 'WF' else 1
    image.save(dest)

with tempfile.TemporaryDirectory(prefix='wg5-synthetic-') as raw:
    fixture=pathlib.Path(raw);project=fixture/'visual-demo';project.mkdir()
    sessions=fixture/'codex/sessions/2026/10/27';sessions.mkdir(parents=True)
    meta={'type':'session_meta','payload':{'id':'visual-goal','session_id':'visual-goal','cwd':str(project),'thread_source':'user'}}
    (sessions/'rollout-visual.jsonl').write_text(json.dumps(meta)+'\n')
    if readability:
        message = '## Delivery plan\nRead src/parser.rs:42 and **keep scope focused**.\n\n- [x] Updated README.md\n- [ ] Check ./tests/parser.rs\n\n> Keep provenance separate from verification.\n\n```rust\nlet parser = read_source("src/parser.rs");\n    validate(parser);\n```\n\nNext: inspect [the spec](docs/WORK.md).'
        records = [
            {'type':'response_item','payload':{'type':'message','role':'assistant','channel':'commentary','content':[{'type':'output_text','text':message}]}},
            {'type':'response_item','payload':{'type':'function_call','name':'exec_command','call_id':'synthetic-command','arguments':json.dumps({'cmd':'cargo test --locked --test parser','justification':'Check the public parser seam'})}},
        ]
        with (sessions/'rollout-visual.jsonl').open('a') as recording:
            for record in records:recording.write(json.dumps(record)+'\n')
    state=fixture/'state'
    def apply(value):
        subprocess.run([executable,'state','--root',str(state),'apply','codex','visual-goal',json.dumps(value)],check=True,stdout=subprocess.DEVNULL)
    owner={'session':{'provider':'codex','session_id':'visual-goal'},'agent_id':'main'}
    apply({'op':'set_goal','title':'WG-5 visual fixture','criteria':[{'id':str(n),'label':label,'weight':1} for n,label in enumerate(['Checkout verified','Payments verified','Reminders verified','Deployment verified','Review verified'])],'reason':'Synthetic terminal QA','provenance':'operator'})
    tasks=[('plan','Plan approved','done',[]),('standard','Standard checkout','done',['plan']),('menus','Choose menus','done',['standard']),('custom','Customize order','done',['menus']),('locations','Delivery locations','done',['menus']),('closing','Close the order','active',['custom','locations']),('stripe','Payment confirmation','planned',['closing']),('reminders','Send reminders','blocked',['stripe']),('corrections','Customer corrections','planned',['stripe']),('docker','Deploy service','planned',['reminders','corrections']),('legacy','Retire old reminder','cancelled',[])]
    for id,label,status,deps in tasks:
        report={'id':id,'label':label,'status':status,'owner':owner,'source':'coordinator · local report','depends_on':deps}
        if id=='closing':
            report.update(purpose='Close the sale safely',next='Review callback expiry')
            if readability:report.update(purpose='Review src/parser.rs:42 and **keep scope focused**.',next='Run `cargo test --locked`, then read [the spec](docs/WORK.md).',change='Source fields are imported deterministically.\n\nKeep provenance separate from outcome evidence.')
        if id=='reminders':report.update(blocker='Await payment contract',next='Review payment event')
        apply({'op':'upsert_work_item','expected_revision':0,'item':report})
    apply({'op':'import_plan','proposal':{'id':'unknown-plan','producer':dict(owner,agent_id='external-worker'),'steps':[{'step':'External review','status':'provider-pending-review'}],'source':'native plan'}})
    try:
        for width,height in [(150,42),(106,48),(60,28)]:
            target=f'view-{width}'
            environment=dict(os.environ,HOME=str(fixture/'home'),CODEX_HOME=str(fixture/'codex'),ZOETROPE_STATE_DIR=str(state),TERM='tmux-256color',NO_COLOR='1')
            launch=shlex.join(['env',*map(lambda pair:f'{pair[0]}={pair[1]}',[(k,environment[k]) for k in ['HOME','CODEX_HOME','ZOETROPE_STATE_DIR','TERM','NO_COLOR']]),executable,'--color','always'])
            tmux('new-session','-d','-s',target,'-x',width,'-y',height,'-c',project,launch)
            until(target,'Current work')
            tmux('send-keys','-t',target,'/')
            tmux('send-keys','-l','-t',target,'WG-5 visual fixture')
            tmux('send-keys','-t',target,'Enter')
            until(target,'Selected: WG-5 visual fixture')
            tmux('send-keys','-t',target,'w')
            until(target,'Close the order')
            time.sleep(.15)
            for mode, key in [('readable','r'),('overview','o')]+([('commands','x')] if readability else []):
                tmux('send-keys','-t',target,key);time.sleep(.15)
                ansi=tmux('capture-pane','-p','-e','-t',target)
                plain=tmux('capture-pane','-p','-t',target)
                stem=f'work-{width}x{height}-{mode}'
                (out/f'{stem}.ansi.txt').write_text(ansi)
                (out/f'{stem}.txt').write_text(plain)
                png(ansi,width,height,out/f'{stem}.png')
    finally:
        try:tmux('kill-server')
        except subprocess.CalledProcessError:pass
print(out)
