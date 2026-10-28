#!/usr/bin/env python3
"""One authorized durable disposable thread; always archive owned thread on exit."""
import json
import os
import time

os.environ['ZOE_PROBE_TRANSCRIPT'] = 'transcript-durable.jsonl'
import probe as p


def main():
    owner = observer = None
    active_turn = None
    try:
        owner, observer = p.Client('durable_owner'), p.Client('durable_observer')
        started = p.ok(owner.call('thread/start', {
            'cwd': p.DISPOSABLE_CWD, 'ephemeral': False, 'sandbox': 'read-only',
            'baseInstructions': 'This is a DISPOSABLE protocol integration probe. Never read/write files, '
                                'run shell commands, access network/apps, or spawn agents. Only invoke '
                                'request_user_input when explicitly asked. Keep all text below 15 words. '
                                'All questions in this probe are harmless.'
        }))
        p.THREAD_ID = started['thread']['id']
        p.ok(owner.call('thread/name/set', {'threadId': p.THREAD_ID,
                                           'name': 'Zoetrope compatibility test - DISPOSABLE 2026-10-23'}))
        p.log('probe', 'metadata', {'threadId': p.THREAD_ID, 'cwd': p.DISPOSABLE_CWD, 'ephemeral': False,
                                  'model': started['model'], 'policy': {k: started.get(k) for k in ['approvalPolicy', 'approvalsReviewer', 'sandbox']}})
        (p.ROOT / 'durable-thread-id.txt').write_text(p.THREAD_ID + '\n')

        # The observer has never joined this thread. Queue CRUD should not start a model.
        def qcall(method, **kw):
            return p.ok(observer.call('thread/queue/' + method, {'threadId': p.THREAD_ID, **kw}))

        q1 = qcall('add', clientUserMessageId='disposable-q1', input=p.text_input('Reply QUEUED_ONE.'))['queuedSubmission']['id']
        q2 = qcall('add', clientUserMessageId='disposable-q2', input=p.text_input('Reply QUEUED_TWO.'))['queuedSubmission']['id']
        qcall('update', queuedSubmissionId=q1, input=p.text_input('Reply QUEUED_ONE_UPDATED.'))
        qcall('reorder', queuedSubmissionIds=[q2, q1])
        listed = qcall('list', limit=10)
        assert [q['id'] for q in listed['data']] == [q2, q1], listed
        time.sleep(1)
        read = p.ok(observer.call('thread/read', {'threadId': p.THREAD_ID, 'includeTurns': False}))
        assert read['thread']['status']['type'] == 'idle', read['thread']['status']
        for qid in [q1, q2]:
            assert qcall('delete', queuedSubmissionId=qid)['deleted'] is True
        assert qcall('list')['data'] == []
        empty = observer.call('thread/queue/start', {'threadId': p.THREAD_ID})
        assert 'error' in empty, empty

        # New turn from an unsubscribed observer. Owner should retain the native question.
        active_turn = p.begin(observer, 'Use request_user_input to ask one harmless question with id probe_choice, '
                              'header Probe, question Choose a disposable label, and options Alpha and Beta. '
                              'After the answer reply only QUESTION_OK.',
                              collaborationMode={'mode': 'plan', 'settings': {'model': started['model'],
                                                'developer_instructions': 'This disposable probe may ask exactly one request_user_input question. '
                                                                          'No plan or file changes are needed. Use no other tools.'}})
        question = owner.wait_message('item/tool/requestUserInput', turn_id=active_turn)
        p.log('probe', 'question_before_explicit_join', {'ownerRequestId': question.get('id'),
              'observerRequestIds': [m.get('id') for m in observer.messages if m.get('method') == 'item/tool/requestUserInput']})
        joined = p.ok(observer.call('thread/resume', {'threadId': p.THREAD_ID, 'excludeTurns': True}))
        p.log('probe', 'join_policy', {k: joined.get(k) for k in ['approvalPolicy', 'approvalsReviewer', 'sandbox', 'model']})
        time.sleep(1)
        p.log('probe', 'question_after_explicit_join', {'ownerRequestId': question.get('id'),
              'observerRequestIds': [m.get('id') for m in observer.messages if m.get('method') == 'item/tool/requestUserInput']})
        owner.send({'id': question['id'], 'result': {'answers': {'probe_choice': {'answers': ['Alpha']}}}})
        assert p.complete(owner, active_turn) == 'completed'
        active_turn = None

        # Explicit native start removes the selected queued item and creates a normal turn.
        queued = qcall('add', clientUserMessageId='disposable-queue-start',
                       input=p.text_input('No questions or tools. Reply only QUEUE_STARTED.'))['queuedSubmission']['id']
        time.sleep(1)
        assert qcall('list')['data'][0]['id'] == queued
        active_turn = qcall('start', queuedSubmissionId=queued)['turn']['id']
        assert p.complete(owner, active_turn) == 'completed'
        active_turn = None
        assert qcall('list')['data'] == []
        p.log('probe', 'summary', {'threadId': p.THREAD_ID,
                                  'ownerMethods': [m['method'] for m in owner.messages],
                                  'observerMethods': [m['method'] for m in observer.messages]})
        print('DURABLE_PROBE_SUCCESS', p.THREAD_ID, flush=True)
    finally:
        if active_turn and owner and p.THREAD_ID:
            try:
                owner.call('turn/interrupt', {'threadId': p.THREAD_ID, 'turnId': active_turn}, timeout=10)
                p.complete(owner, active_turn)
            except Exception as exc:
                p.log('probe', 'cleanup_error', {'error': str(exc)})
        if owner and p.THREAD_ID:
            try:
                p.ok(owner.call('thread/archive', {'threadId': p.THREAD_ID}, timeout=20))
            except Exception as exc:
                p.log('probe', 'archive_error', {'error': str(exc)})
        for client in [observer, owner]:
            if client:
                client.close()
        p.LOG.close()


if __name__ == '__main__':
    main()
