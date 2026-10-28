const assert = require('node:assert/strict');
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const vm = require('node:vm');
const { pathToFileURL, fileURLToPath } = require('node:url');
const { test } = require('node:test');

// Adapter for the external VS Code host. The extension is loaded through its
// packaged entry point and only its public activation/URI interface is exercised.
function editorHost(t, options = {}) {
    const root = fs.mkdtempSync(path.join(os.tmpdir(), 'zoetrope-preview-'));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    let handler;
    const host = {
        activeDocument: undefined,
        readerDocuments: [],
        information: [],
        errors: [],
        Uri: {
            parse(value) {
                const uri = new URL(value);
                return {
                    scheme: uri.protocol.slice(0, -1),
                    authority: uri.host,
                    path: decodeURIComponent(uri.pathname),
                    // VS Code Uri.parse decodes components once, including query.
                    // https://github.com/microsoft/vscode-uri/blob/main/src/uri.ts
                    query: decodeURIComponent(uri.search.slice(1)),
                    fragment: uri.hash.slice(1),
                    fsPath: uri.protocol === 'file:' ? fileURLToPath(uri) : undefined,
                    toString: () => uri.toString(),
                };
            },
        },
        workspace: {
            async openTextDocument(uri) {
                return { uri, text: fs.readFileSync(uri.fsPath, 'utf8') };
            },
        },
        window: {
            registerUriHandler(value) {
                handler = value;
                return { dispose() {} };
            },
            async showTextDocument(document) {
                host.activeDocument = document;
                return { document };
            },
            async showInformationMessage(message) { host.information.push(message); },
            async showErrorMessage(message) { host.errors.push(message); },
        },
        extensions: {
            getExtension(id) {
                assert.equal(id.toLowerCase(), 'anandshah.mdreader');
                if (options.readerInstalled === false) return undefined;
                return { async activate() {
                    if (options.activationFailure) throw new Error('reader activation failed');
                } };
            },
        },
        commands: {
            async getCommands() { return options.commandAvailable === false ? [] : ['mdreader.fullRender']; },
            async executeCommand(command) {
                assert.equal(command, 'mdreader.fullRender');
                if (options.readerInstalled === false || options.commandAvailable === false) {
                    throw new Error('command unavailable');
                }
                if (options.renderFailure) throw new Error('reader command failed');
                host.readerDocuments.push(host.activeDocument);
            },
        },
    };
    const manifest = JSON.parse(fs.readFileSync(path.join(__dirname, '../package.json')));
    assert.ok(manifest.activationEvents.includes('onUri'));
    assert.equal(`${manifest.publisher}.${manifest.name}`, 'wagar.zoetrope-preview');
    const extension = { exports: {} };
    vm.runInNewContext(fs.readFileSync(path.join(__dirname, '..', manifest.main), 'utf8'), {
        module: extension,
        exports: extension.exports,
        require(name) { assert.equal(name, 'vscode'); return host; },
        URLSearchParams,
    });
    const context = { subscriptions: [] };
    extension.exports.activate(context);
    assert.equal(context.subscriptions.length, 1);
    return {
        host,
        root,
        dispatch(uri) { return handler.handleUri(host.Uri.parse(uri)); },
        open(file) {
            const uri = `vscode://wagar.zoetrope-preview/open?uri=${encodeURIComponent(pathToFileURL(file).href)}`;
            return handler.handleUri(host.Uri.parse(uri));
        },
    };
}

test('the public URI opens the exact selected document in MD Reader', async (t) => {
    const editor = editorHost(t);
    const selected = path.join(editor.root, '报告 space $(touch NEVER); & #+%.md');
    fs.writeFileSync(selected, '# The selected document');

    await editor.open(selected);

    assert.equal(editor.host.readerDocuments.length, 1);
    assert.equal(editor.host.readerDocuments[0].uri.fsPath, selected);
    assert.equal(editor.host.readerDocuments[0].text, '# The selected document');
    assert.deepEqual(editor.host.errors, []);
});

test('the public bridge rejects commands, remote resources, and non-Markdown targets', async (t) => {
    const editor = editorHost(t);
    const targets = [
        'command:workbench.action.openSettings',
        'https://example.invalid/document.md',
        'file://remote-host/share/document.md',
        pathToFileURL(path.join(editor.root, 'script.command')).href,
        `${pathToFileURL(path.join(editor.root, 'report.md')).href}?other=file`,
        `${pathToFileURL(path.join(editor.root, 'report.md')).href}#section`,
    ];
    for (const target of targets) {
        await editor.dispatch(`vscode://wagar.zoetrope-preview/open?uri=${encodeURIComponent(target)}`);
    }

    assert.equal(editor.host.errors.length, targets.length);
    assert.ok(editor.host.errors.every(message => message.includes('Zoetrope could not open document')));
    assert.equal(editor.host.activeDocument, undefined);
    assert.deepEqual(editor.host.readerDocuments, []);
});

test('an unavailable MD Reader leaves the selected file open with a clear explanation', async (t) => {
    for (const options of [{ readerInstalled: false }, { commandAvailable: false }]) {
        const editor = editorHost(t, options);
        const selected = path.join(editor.root, 'plain.md');
        fs.writeFileSync(selected, '# Plain fallback');

        await editor.open(selected);

        assert.equal(editor.host.activeDocument.uri.fsPath, selected);
        assert.deepEqual(editor.host.readerDocuments, []);
        assert.ok(editor.host.information.some(message => message.includes('MD Reader is unavailable')));
        assert.deepEqual(editor.host.errors, []);
    }
});

test('concurrent preview requests keep each reader invocation on its own selected file', async (t) => {
    const editor = editorHost(t);
    const first = path.join(editor.root, 'first.md');
    const second = path.join(editor.root, 'second.md');
    fs.writeFileSync(first, '# First');
    fs.writeFileSync(second, '# Second');

    await Promise.all([editor.open(first), editor.open(second)]);

    assert.deepEqual(editor.host.readerDocuments.map(document => document.uri.fsPath), [first, second]);
    assert.deepEqual(editor.host.readerDocuments.map(document => document.text), ['# First', '# Second']);
});

test('MD Reader activation and command failures fall back to the selected plain document', async (t) => {
    for (const options of [{ activationFailure: true }, { renderFailure: true }]) {
        const editor = editorHost(t, options);
        const selected = path.join(editor.root, 'fallback.md');
        fs.writeFileSync(selected, '# Read this result');

        await editor.open(selected);

        assert.equal(editor.host.activeDocument?.uri.fsPath, selected);
        assert.deepEqual(editor.host.readerDocuments, []);
        assert.ok(editor.host.information.some(message => message.includes('MD Reader is unavailable')));
        assert.deepEqual(editor.host.errors, []);
    }
});
