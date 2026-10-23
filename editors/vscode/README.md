# Zoetrope document preview bridge

Local VS Code extension `wagar.zoetrope-preview`. An explicit document-open action in Zoetrope requests MD Reader for the selected existing `.md` or `.markdown` file. No network service, model call, automatic transcript opening, or renderer preference change is involved.

The native launcher checks for the bridge with `code --list-extensions` and dispatches one URI through the OS opener:

```text
vscode://wagar.zoetrope-preview/open?uri=<percent-encoded-file-URI>
```

The handler accepts only local Markdown file URIs, opens the selected document, activates `AnandShah.mdreader`, shows that exact document, and invokes `mdreader.fullRender`. Concurrent requests are serialized. An unavailable or failed reader leaves the selected document open in the plain editor with an explanation. A missing document produces a visible error. No arbitrary command or remote-resource URI is accepted.

If the bridge is absent or URI dispatch fails, the native launcher requests the file with `code --reuse-window <absolute-path>`. Paths are separate process arguments, including spaces, Unicode and shell metacharacters. Selected symlink spelling is retained. Relative input is made absolute before dispatch, so the editor's working directory does not change its target.

Zoetrope reports **Requested VS Code preview** or **Requested VS Code file**. Successful OS dispatch does not establish that an editor window or rendered panel appeared. Extension listing establishes installation, not whether the bridge is enabled in the current window/profile. There is no render acknowledgment protocol. macOS with stable VS Code is the initial integration target.

## Build and install locally

Python 3 standard library is sufficient; no npm install or Marketplace publication is needed.

```bash
python3 editors/vscode/package_vsix.py
code --install-extension editors/vscode/dist/zoetrope-preview-0.1.0.vsix
```

Keep the user's existing MD Reader installed/enabled. Reload VS Code if prompted. The bridge does not install MD Reader or change default Markdown preferences. A real preview check is required after installation; the test host below does not prove compatibility with every VS Code/reader configuration.

## Verification

```bash
node --test editors/vscode/test/preview.test.cjs
cargo test --locked --test document_launch
```

The Node tests activate the packaged extension entry point in an external VS Code host adapter and send requests through the registered public URI handler. They read real fixture files and observe the document received by the MD Reader command. Native tests execute recording editor/opener processes and independently decode the URI at that external seam.

API references: [extension URI activation](https://code.visualstudio.com/api/references/activation-events#onUri), [commands](https://code.visualstudio.com/api/extension-guides/command), [VS Code CLI](https://code.visualstudio.com/docs/configure/command-line).
