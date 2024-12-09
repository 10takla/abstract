import path from 'path';
import vscode, { workspace } from 'vscode';
import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
	const outputChannel = vscode.window.createOutputChannel('Abstract');
	outputChannel.show(true);
	outputChannel.appendLine('Started');
	
	const serverPath = process.env.LSP_SERVER_PATH || context.asAbsolutePath(
        path.join('..', '..', 'target', 'debug', 'lsp-server')
    );
	outputChannel.appendLine(serverPath);
	outputChannel.appendLine(process.cwd());
	let serverOptions: ServerOptions = {
		run: { command: serverPath, transport: TransportKind.stdio },
		debug: { command: serverPath, transport: TransportKind.stdio }
	};
	let clientOptions: LanguageClientOptions = {
		documentSelector: [{ scheme: 'file', language: "abstract" }],
		outputChannel: outputChannel,
		traceOutputChannel: outputChannel,
		synchronize: {
			fileEvents: workspace.createFileSystemWatcher('**/.abs')
		}
	};

	client = new LanguageClient(
		'abstract',
		'Abstract Language Server',
		serverOptions,
		clientOptions
	);

	client.start();
}

export function deactivate(): Thenable<void> | undefined {
	return client ? client.stop() : undefined;
}