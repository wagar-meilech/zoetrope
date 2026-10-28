#!/usr/bin/env python3
"""Continue only our recorded durable disposable thread, then archive it."""
import os
import time
os.environ['ZOE_PROBE_TRANSCRIPT'] = 'transcript-durable-continued.jsonl'
import probe as p


def main():
    owner = observer = None
    active_turn = None
    try:
        owner, observer = p.Client('joined_owner'), p.Client('joined_observer')
        p.THREAD_ID = (p.ROOT / 'durable-thread-id.txt').read_text().strip()

        def qcall(method, **kw):
            return p.ok(observer.call('thread/queue/' + method, {'threadId': p.THREAD_ID, **kw}))

        p.ok(owner.call('thread/unarchive', {'threadId': p.THREAD_ID}))
        # Clear only the pending inputs left in our own disposable thread.
        for entry in qcall('list')['data']:
            qcall('delete', queuedSubmissionId=entry['id'])
        resumed = p.ok(owner.call('thread/resume', {'threadId': p.THREAD_ID, 'excludeTurns': True}))
        p.log('probe', 'resume_policy', {k: resumed.get(k) for k in ['approvalPolicy', 'approvalsReviewer', 'sandbox', 'model']})
        model = resumed['model']
        active_turn = p.begin(owner, 'Use request_user_input to ask one harmless question with id probe_choice, '
                              'header Probe, question Choose a disposable label, and options Alpha and Beta. '
                              'After the answer reply only QUESTION_OK.',
                              collaborationMode={'mode': 'plan', 'settings': {'model': model,
                                                'developer_instructions': 'This disposable probe may ask request_user_input questions. '
                                                                          'No plan or file changes are needed. Use no other tools.'}})
        question = owner.wait_message('item/tool/requestUserInput', turn_id=active_turn)

        # Active native question blocks generation while queue CRUD remains inexpensive.
        q1 = qcall('add', clientUserMessageId='continued-q1', input=p.text_input('Reply QUEUED_ONE.'))['queuedSubmission']['id']
        q2 = qcall('add', clientUserMessageId='continued-q2', input=p.text_input('Reply QUEUED_TWO.'))['queuedSubmission']['id']
        qcall('update', queuedSubmissionId=q1, input=p.text_input('Reply QUEUED_ONE_UPDATED.'))
        qcall('reorder', queuedSubmissionIds=[q2, q1])
        listed = qcall('list', limit=10)
        assert [q['id'] for q in listed['data']] == [q2, q1], listed
        for qid in [q1, q2]:
            assert qcall('delete', queuedSubmissionId=qid)['deleted'] is True
        assert qcall('list')['data'] == []
        empty = observer.call('thread/queue/start', {'threadId': p.THREAD_ID})
        assert 'error' in empty, empty

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

        # Queue add on idle starts automatically. A subsequent native question probes both joined clients.
        before = len(owner.messages)
        qcall('add', clientUserMessageId='continued-auto-queue', input=p.text_input(
            'Use request_user_input to ask one harmless question with id queue_choice, header Queue, '
            'question Choose a disposable label, and options Alpha and Beta. After the answer reply only QUEUE_OK.'))
        start = owner.wait_message('turn/started', after=before)
        active_turn = start['params']['turn']['id']
        question2 = owner.wait_message('item/tool/requestUserInput', after=before, turn_id=active_turn)
        time.sleep(0.7)
        p.log('probe', 'question_created_after_join', {'ownerRequestId': question2.get('id'),
              'observerRequestIds': [m.get('id') for m in observer.messages if m.get('method') == 'item/tool/requestUserInput'
                                     and m.get('params', {}).get('turnId') == active_turn]})
        owner.send({'id': question2['id'], 'result': {'answers': {'queue_choice': {'answers': ['Alpha']}}}})
        assert p.complete(owner, active_turn) == 'completed'
        active_turn = None
        assert qcall('list')['data'] == []
        p.log('probe', 'summary', {'threadId': p.THREAD_ID,
                                  'ownerMethods': [m['method'] for m in owner.messages],
                                  'observerMethods': [m['method'] for m in observer.messages]})
        print('CONTINUED_PROBE_SUCCESS', p.THREAD_ID, flush=True)
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
