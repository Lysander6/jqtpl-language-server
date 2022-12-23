import { ExtensionContext } from "vscode";

import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind,
} from "vscode-languageclient/node";

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  // before packaging, move built server to base directory of client extension
  const serverModule = context.asAbsolutePath("server.js");

  const serverOptions: ServerOptions = {
    run: { module: serverModule, transport: TransportKind.ipc },
    debug: { module: serverModule, transport: TransportKind.ipc },
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: "file", language: "jqtpl" }],
  };

  client = new LanguageClient(
    "jqtpl-language-server",
    "JQTPL Langauge Server",
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | void {
  if (!client) {
    return undefined;
  }

  return client.stop();
}
