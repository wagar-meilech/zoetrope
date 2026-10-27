Provide a small local URI bridge to open selected Markdown documents in the user's installed reader. Keep the opener a native argument-based invocation, validate selected files, distinguish dispatch from render acknowledgment, and provide a plain-editor fallback. Do not publish to a marketplace or change global renderer preferences.

Acceptance: the public launch interface carries the exact selected file; spaces, Unicode and shell-like characters remain data; unavailable files report a clear error; the bridge opens the document and invokes MD Reader through its public command. Verify one real preview on this Mac after installing the local bridge.

## Context

Spec: https://github.com/wagar-meilech/zoetrope/issues/1

Integration branch: `feat/goal-console`.

## Dependencies

- No blocking tickets. The preview bridge takes an explicit selected document and can be implemented independently; integration with discovered document records occurs in ZOE-4/ZOE-6.

## Implementation workflow

Use the approved public console and executable/external integration seams. Follow vertical TDD: observe red, implement one behavior, observe green. Work in an isolated worktree based on the latest integration branch and merge that tip before handoff. Do not alter unrelated live Codex sessions.
