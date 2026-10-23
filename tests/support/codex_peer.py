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

init = request('initialize', result={'userAgent':'codex/0.164.0','platformFamily':'unix','platformOs':'macos'})
assert init['params']['capabilities']['experimentalApi'] is True
assert read()['method'] == 'initialized'
if case != 'loaded':
    request('thread/read', {'threadId':'child-thread','includeTurns':False}, {'thread':{'id':'child-thread','status':({'type':'idle'} if case == 'idle' else {'type':'active','activeFlags':[]}),'name':'Parser','cwd':'/project','canAcceptDirectInput':case not in ('no-direct','restricted-stop')}})
    if case != 'idle':
        request('thread/turns/list', {'threadId':'child-thread','limit':1,'sortDirection':'desc','itemsView':'notLoaded'}, {'data':[{'id':'turn-current','status':'inProgress'}],'nextCursor':None})
if case == 'guide':
    request('turn/steer', {'threadId':'child-thread','expectedTurnId':'turn-current','input':[{'type':'text','text':'Keep the public interface'}]}, {'turnId':'turn-current'})


if case == 'queue':
    request('thread/queue/add', {'threadId':'child-thread','clientUserMessageId':'zoe-item-17','input':[{'type':'text','text':'Do the docs later'}]}, {'queuedSubmission':{'id':'queued-17'}})
if case in ('stop','restricted-stop'):
    request('turn/interrupt', {'threadId':'child-thread','turnId':'turn-current'}, {})

if case == 'idle':
    request('turn/start', {'threadId':'child-thread','input':[{'type':'text','text':'Continue the work'}]}, {'turn':{'id':'new-turn','status':'inProgress'}})

if case in ('join','answer','resolved-answer'):
    request('thread/resume', {'threadId':'child-thread','excludeTurns':True}, {'thread':{'id':'child-thread'}})
    send({'id':7,'method':'item/tool/requestUserInput','params':{'threadId':'child-thread','turnId':'turn-current','questions':[{'id':'choice','header':'Label','question':'Choose a label','options':[{'label':'Alpha','description':'Choose Alpha'},{'label':'Beta','description':'Choose Beta'}],'isOther':True,'isSecret':False}]}})
    if case in ('join','resolved-answer'):
        send({'method':'serverRequest/resolved','params':{'threadId':'child-thread','requestId':7}})

if case in ('answer','resolved-answer'):
    request('thread/read', {'threadId':'child-thread','includeTurns':False}, {'thread':{'id':'child-thread','status':{'type':'active','activeFlags':['waitingOnUserInput']}}})
    request('thread/turns/list', {'threadId':'child-thread','limit':1,'sortDirection':'desc','itemsView':'notLoaded'}, {'data':[{'id':'turn-current','status':'inProgress'}]})
    if case == 'answer':
        assert read() == {'id':7,'result':{'answers':{'choice':{'answers':['Beta']}}}}
    else:
        try:
            unexpected = read()
            raise AssertionError(unexpected)
        except EOFError:
            pass

if case == 'loaded':
    request('thread/loaded/list', {'limit':128}, {'data':['quiet-working'],'nextCursor':'page-two'})
    request('thread/loaded/list', {'limit':128,'cursor':'page-two'}, {'data':['waiting-question'],'nextCursor':None})

if case in ('reject','disconnect'):
    obj = read()
    assert obj['method'] == 'turn/steer'
    assert obj['params'] == {'threadId':'child-thread','expectedTurnId':'turn-current','input':[{'type':'text','text':'Keep the public interface'}]}
    if case == 'reject':
        send({'id':obj['id'],'error':{'code':-32600,'message':'expected turn-current but actual active turn is newer-turn'}})
    else:
        conn.shutdown(socket.SHUT_RDWR)
if case in ('stale','no-direct'):
    try:
        unexpected = read()
        raise AssertionError(unexpected)
    except EOFError:
        pass

if case == 'wrong-ack':
    request('turn/steer', {'threadId':'child-thread','expectedTurnId':'turn-current','input':[{'type':'text','text':'Keep the public interface'}]}, {'turnId':'different-turn'})
conn.close()
s.close()
