#!/usr/bin/env python3
"""Disposable shared-runtime compatibility probe; never accepts existing thread IDs."""
import json
import os
import pathlib
import queue
import socket
import tempfile
import threading
import time
import uuid

import websocket

ROOT = pathlib.Path(__file__).resolve().parent
SOCKET = '/Users/melikhov/.codex/app-server-control/app-server-control.sock'
LOG = (ROOT / os.environ.get('ZOE_PROBE_TRANSCRIPT', 'transcript-control.jsonl')).open('w')
LOCK = threading.Lock()
THREAD_ID = None
DISPOSABLE_CWD = tempfile.mkdtemp(prefix='zoe-DISPOSABLE-control-')


def log(client, direction, obj):
    with LOCK:
        LOG.write(json.dumps({'time': time.time(), 'client': client,
                              'direction': direction, 'message': obj}) + '\n')
        LOG.flush()


def scoped(message):
    params = message.get('params', {})
    thread = params.get('thread', {})
    return ((THREAD_ID and params.get('threadId') == THREAD_ID) or
            (thread.get('cwd') == DISPOSABLE_CWD))


class Client:
    def __init__(self, name):
        self.name = name
        self.messages = []
        self.responses = {}
        self.cv = threading.Condition()
        self.counter = 0
        self.closed = False
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.settimeout(10)
        sock.connect(SOCKET)
        self.ws = websocket.create_connection('ws://localhost/', socket=sock, timeout=1)
        self.reader = threading.Thread(target=self.read, daemon=True)
        self.reader.start()
        self.call('initialize', {'clientInfo': {'name': 'zoe_disposable_' + name,
                    'title': 'Zoetrope DISPOSABLE Compatibility ' + name,
                    'version': '0.0.0'}, 'capabilities': {'experimentalApi': True}})
        self.send({'method': 'initialized', 'params': {}})

    def send(self, obj):
        log(self.name, 'send', obj)
        self.ws.send(json.dumps(obj))

    def read(self):
        while not self.closed:
            try:
                raw = self.ws.recv()
                if not raw:
                    return
                obj = json.loads(raw)
            except websocket.WebSocketTimeoutException:
                continue
            except Exception as exc:
                if not self.closed:
                    log(self.name, 'reader_error', {'error': str(exc)})
                return
            with self.cv:
                if 'id' in obj and ('result' in obj or 'error' in obj):
                    self.responses[obj['id']] = obj
                    log(self.name, 'receive', obj)
                elif scoped(obj):
                    self.messages.append(obj)
                    log(self.name, 'receive', obj)
                # Ignore notifications/requests about all other sessions.
                self.cv.notify_all()

    def call(self, method, params, timeout=30):
        self.counter += 1
        rid = self.name + '-' + str(self.counter)
        self.send({'id': rid, 'method': method, 'params': params})
        deadline = time.monotonic() + timeout
        with self.cv:
            while rid not in self.responses:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise TimeoutError(method)
                self.cv.wait(remaining)
            result = self.responses.pop(rid)
        print(self.name, method, json.dumps(result.get('error', result.get('result', {})))[:900], flush=True)
        return result

    def wait_message(self, method, after=0, timeout=90, turn_id=None):
        deadline = time.monotonic() + timeout
        with self.cv:
            while True:
                for msg in self.messages[after:]:
                    if msg.get('method') != method:
                        continue
                    params = msg.get('params', {})
                    if turn_id and params.get('turnId', params.get('turn', {}).get('id')) != turn_id:
                        continue
                    return msg
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise TimeoutError(method)
                self.cv.wait(remaining)

    def close(self):
        self.closed = True
        self.ws.close()


def text_input(text):
    return [{'type': 'text', 'text': text}]


def ok(response):
    if 'error' in response:
        raise RuntimeError(response['error'])
    return response['result']


def begin(client, text, **kwargs):
    return ok(client.call('turn/start', {'threadId': THREAD_ID, 'input': text_input(text), **kwargs}))['turn']['id']


def complete(owner, turn_id):
    msg = owner.wait_message('turn/completed', turn_id=turn_id)
    print('COMPLETED', json.dumps(msg), flush=True)
    return msg['params']['turn']['status']


def main():
    global THREAD_ID
    owner = observer = None
    active_turn = None
    try:
        owner = Client('owner')
        observer = Client('observer')
        started = ok(owner.call('thread/start', {
            'cwd': DISPOSABLE_CWD, 'ephemeral': True, 'sandbox': 'read-only',
            'baseInstructions': 'This is a DISPOSABLE protocol integration probe. Never read or write files, '
                                'run shell commands, access network/apps, or spawn agents. Only invoke '
                                'probe_gate or request_user_input when explicitly asked. Keep all text below 15 words. '
                                'Follow later user steering. All tools and questions in this probe are harmless.',
            'dynamicTools': [{'type': 'function', 'name': 'probe_gate',
                              'description': 'Harmless disposable integration gate. Call only when instructed; has no side effects.',
                              'inputSchema': {'type': 'object', 'properties': {}, 'additionalProperties': False}}]
        }))
        THREAD_ID = started['thread']['id']
        log('probe', 'metadata', {'threadId': THREAD_ID, 'cwd': DISPOSABLE_CWD, 'ephemeral': True,
                                 'model': started['model'], 'policy': {k: started.get(k) for k in ['approvalPolicy', 'approvalsReviewer', 'sandbox']}})
        (ROOT / 'thread-id.txt').write_text(THREAD_ID + '\n')

        # Owner starts work; observer has never resumed/started/subscribed to this thread.
        active_turn = begin(owner, 'Call probe_gate once, wait for its result, then reply GATE_OK unless later steering requests another token.')
        gate = owner.wait_message('item/tool/call', turn_id=active_turn)
        ok(observer.call('thread/read', {'threadId': THREAD_ID, 'includeTurns': False}))
        stale = observer.call('turn/steer', {'threadId': THREAD_ID, 'expectedTurnId': str(uuid.uuid4()),
                                           'input': text_input('Reply STALE_SHOULD_NEVER_APPEAR.')})
        if 'error' not in stale:
            raise RuntimeError('Stale steer unexpectedly accepted')
        ok(observer.call('turn/steer', {'threadId': THREAD_ID, 'expectedTurnId': active_turn,
                                      'input': text_input('After probe_gate returns, reply only STEER_ACCEPTED.')}))
        log('probe', 'unsubscribed_observer_events_before_start', {'methods': [m['method'] for m in observer.messages]})
        owner.send({'id': gate['id'], 'result': {'success': True, 'contentItems': [{'type': 'inputText', 'text': 'GATE_RELEASED'}]}})
        complete(owner, active_turn)
        active_turn = None

        # Interrupt exact current turn from the still-unsubscribed observer.
        before = len(owner.messages)
        active_turn = begin(owner, 'Call probe_gate once, wait for its result, then reply INTERRUPT_NOT_EXPECTED.')
        owner.wait_message('item/tool/call', after=before, turn_id=active_turn)
        ok(observer.call('turn/interrupt', {'threadId': THREAD_ID, 'turnId': active_turn}))
        if complete(owner, active_turn) != 'interrupted':
            raise RuntimeError('Exact interrupt did not interrupt')
        active_turn = None

        # Idle feedback starts a new turn by ID without thread/resume. Also make a harmless native question.
        active_turn = begin(observer, 'Use request_user_input to ask one harmless question with id probe_choice, '
                            'header Probe, question Choose a disposable label, and two options Alpha and Beta. '
                            'After the answer reply only QUESTION_OK.',
                            collaborationMode={'mode': 'plan', 'settings': {'model': started['model'],
                                                'developer_instructions': 'This disposable probe may ask exactly one request_user_input question. '
                                                                          'No plan or file changes are needed. Use no other tools.'}})
        question = owner.wait_message('item/tool/requestUserInput', turn_id=active_turn)
        observer_before_join = [m for m in observer.messages if m.get('method') == 'item/tool/requestUserInput']
        log('probe', 'question_before_explicit_join', {'ownerRequestId': question.get('id'),
                                                     'observerRequestIds': [m.get('id') for m in observer_before_join]})
        # Installed 0.164.0 rejects ephemeral rejoin because no rollout exists.
        joined = observer.call('thread/resume', {'threadId': THREAD_ID, 'excludeTurns': True})
        if 'error' in joined and 'no rollout found' not in joined['error']['message']:
            raise RuntimeError(joined['error'])
        time.sleep(0.7)
        observer_questions = [m for m in observer.messages if m.get('method') == 'item/tool/requestUserInput']
        log('probe', 'question_after_explicit_join', {'ownerRequestId': question.get('id'),
                                                    'observerRequestIds': [m.get('id') for m in observer_questions]})
        owner.send({'id': question['id'], 'result': {'answers': {'probe_choice': {'answers': ['Alpha']}}}})
        complete(owner, active_turn)
        active_turn = None

        log('probe', 'summary', {'threadId': THREAD_ID, 'ownerMethods': [m['method'] for m in owner.messages],
                                 'observerMethods': [m['method'] for m in observer.messages]})
        print('PROBE_SUCCESS', THREAD_ID, flush=True)
    finally:
        if active_turn and owner and THREAD_ID:
            try:
                owner.call('turn/interrupt', {'threadId': THREAD_ID, 'turnId': active_turn}, timeout=10)
            except Exception as exc:
                log('probe', 'cleanup_error', {'error': str(exc)})
        for client in [observer, owner]:
            if client:
                client.close()
        LOG.close()


if __name__ == '__main__':
    main()
