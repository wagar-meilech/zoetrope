"""Independent stdlib WebSocket protocol peer for the executable seam."""
import base64
import hashlib
import json
import pathlib
import socket
import struct
import sys

path, case = sys.argv[1:]
s = socket.socket(socket.AF_UNIX)
s.bind(path)
s.listen(1)
pathlib.Path(path + '.ready').touch()
s.settimeout(8)
conn, _ = s.accept()
conn.settimeout(8)
headers = bytearray()
while not headers.endswith(b'\r\n\r\n'):
    headers.extend(conn.recv(1))
key = next(line.split(b':', 1)[1].strip() for line in headers.split(b'\r\n') if line.lower().startswith(b'sec-websocket-key:'))
accept = base64.b64encode(hashlib.sha1(key + b'258EAFA5-E914-47DA-95CA-C5AB0DC85B11').digest())
conn.sendall(b'HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: ' + accept + b'\r\n\r\n')

def exact(n):
    data = bytearray()
    while len(data) < n:
        chunk = conn.recv(n-len(data))
        if not chunk:
            raise EOFError()
        data.extend(chunk)
    return bytes(data)

def read():
    first, second = exact(2)
    assert first == 0x81, first
    assert second & 0x80, 'client must mask'
    n = second & 127
    if n == 126:
        n = struct.unpack('!H', exact(2))[0]
    elif n == 127:
        n = struct.unpack('!Q', exact(8))[0]
    mask = exact(4)
    return json.loads(bytes(x ^ mask[i % 4] for i,x in enumerate(exact(n))))

def send(obj):
    data = json.dumps(obj).encode()
    prefix = bytes([0x81, len(data)]) if len(data) < 126 else bytes([0x81, 126]) + struct.pack('!H',len(data))
    conn.sendall(prefix + data)

def request(method, params=None, result=None):
    obj = read()
    assert obj['method'] == method, obj
    if params is not None:
        assert obj['params'] == params, obj
    send({'id':obj['id'], 'result':result or {}})
    return obj


request('initialize', result={'userAgent':'codex/0.164.0','platformFamily':'unix','platformOs':'macos'})
assert read()['method'] == 'initialized'
seen=[]
try:
 while True:
  obj=read();method=obj.get('method');params=obj.get('params',{})
  if method=='thread/loaded/list': result={'data':([f'idle-{i}' for i in range(70)]+['root-goal'] if case=='many' else ['root-goal','child-thread']),'nextCursor':None}
  elif method=='thread/read':
   tid=params['threadId'];assert tid in ('root-goal','child-thread') or (case=='many' and tid.startswith('idle-'));assert params=={'threadId':tid,'includeTurns':False};result={'thread':{'id':tid,'name':('Parser' if tid=='child-thread' else 'Parser goal'),'cwd':'/projects/parser','status':{'type':'active','activeFlags':(['waitingOnUserInput'] if case=='observe' or (case=='many' and tid=='root-goal') else [])},'canAcceptDirectInput':not (case=='restricted' and tid=='child-thread')}}
   if case=='many' and tid!='root-goal': result['thread']['status']={'type':'idle'}
  elif method=='thread/turns/list':
   tid=params['threadId'];result={'data':[{'id':('child-turn' if tid=='child-thread' else 'root-turn'),'status':'inProgress'}],'nextCursor':None}
  elif method=='thread/resume' and case=='question':
   assert params=={'threadId':'root-goal','excludeTurns':True},obj
   seen.append('join')
   send({'id':7,'method':'item/tool/requestUserInput','params':{'threadId':'root-goal','turnId':'root-turn','itemId':'native-question-call','questions':[{'id':'choice','header':'Label','question':'Choose a label','options':[{'label':'Alpha','description':'First choice'},{'label':'Beta','description':'Second choice'}],'isOther':True,'isSecret':False}]}})
   result={'thread':{'id':'root-goal'}}
  elif method is None and case=='question':
   assert obj=={'id':7,'result':{'answers':{'choice':{'answers':['Beta']}}}},obj
   seen.append('answer');send({'method':'serverRequest/resolved','params':{'threadId':'root-goal','requestId':7}});continue
  elif method=='turn/steer':
   assert params=={'threadId':'child-thread','expectedTurnId':'child-turn','input':[{'type':'text','text':'Keep the public interface'}]},obj
   seen.append('guide')
   if case=='pending' and seen.count('guide')==1:
    import time
    pathlib.Path(path+'.sending').touch()
    deadline=time.monotonic()+8
    while not pathlib.Path(path+'.release').exists():
     assert time.monotonic()<deadline,'operator did not release acknowledgment'
     time.sleep(.01)
   result={'turnId':'child-turn'}
  elif method=='thread/queue/add':
   assert params['threadId']=='child-thread' and params['input']==[{'type':'text','text':'Do docs later'}],obj
   assert params['clientUserMessageId'];seen.append('queue');result={'queuedSubmission':{'id':'queued-ui'}}
  elif method=='turn/interrupt':
   assert params=={'threadId':'child-thread','turnId':'child-turn'},obj
   seen.append('stop');result={}
  else: raise AssertionError(obj)
  send({'id':obj['id'],'result':result})
except EOFError:
 pass
assert seen==({'question':['join','answer'],'observe':[],'many':[],'pending':['guide','guide'],'restricted':['stop']}.get(case,['guide','queue','stop'])),seen
conn.close();s.close()
