const vscode = require('vscode');

function activate(context) {
    let pending = Promise.resolve();
    context.subscriptions.push(vscode.window.registerUriHandler({
        handleUri(uri) {
            const request = pending.then(async () => {
                try {
                    // VS Code already percent-decodes Uri.query once. Decode the
                    // nested file URI only in Uri.parse, preserving #, +, %, &.
                    if (uri.path !== '/open' || !uri.query.startsWith('uri=')) {
                        throw new Error('Select one local Markdown document.');
                    }
                    const target = vscode.Uri.parse(uri.query.slice(4), true);
                    if (target.scheme !== 'file' || target.authority || target.query || target.fragment
                        || !/\.(md|markdown)$/i.test(target.path)) {
                        throw new Error('Select a local Markdown file (.md or .markdown).');
                    }
                    const document = await vscode.workspace.openTextDocument(target);
                    let readerProblem;
                    try {
                        const reader = vscode.extensions.getExtension('AnandShah.mdreader');
                        if (!reader) throw new Error('install or enable AnandShah.mdreader');
                        await reader.activate();
                        if (!(await vscode.commands.getCommands(true)).includes('mdreader.fullRender')) {
                            throw new Error('mdreader.fullRender is not registered');
                        }
                    } catch (error) {
                        readerProblem = error.message;
                    }
                    await vscode.window.showTextDocument(document, { preview: false });
                    if (!readerProblem) {
                        try {
                            await vscode.commands.executeCommand('mdreader.fullRender');
                            return;
                        } catch (error) {
                            readerProblem = error.message;
                            await vscode.window.showTextDocument(document, { preview: false });
                        }
                    }
                    await vscode.window.showInformationMessage(`Opened the selected file in VS Code. MD Reader is unavailable: ${readerProblem}. Use MD Reader: Full Render after resolving this.`);
                } catch (error) {
                    await vscode.window.showErrorMessage(`Zoetrope could not open document: ${error.message}`);
                }
            });
            pending = request.catch(() => {});
            return request;
        },
    }));
}

module.exports = { activate };
