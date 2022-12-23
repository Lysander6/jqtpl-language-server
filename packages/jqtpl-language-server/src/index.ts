#!/usr/bin/env node

import {
  CompletionItem,
  CompletionItemKind,
  createConnection,
  InitializeResult,
  InsertTextFormat,
  ProposedFeatures,
  // TextDocuments,
} from "vscode-languageserver/node";

// import { TextDocument } from 'vscode-languageserver-textdocument';

// TODO: `chmod +x` built `index.js`

const connection = createConnection(ProposedFeatures.all);
// const documents = new TextDocuments(TextDocument);

connection.onInitialize((_params) => {
  const result: InitializeResult = {
    capabilities: {
      completionProvider: {
        resolveProvider: true,
        triggerCharacters: ["{{"],
      },
    },
  };

  return result;
});

connection.onCompletion((_params) => {
  const completions: CompletionItem[] = [
    {
      label: "if",
      kind: CompletionItemKind.Text,
      insertText: "{{if $1}}$2{{/if}}$0",
      insertTextFormat: InsertTextFormat.Snippet,
      data: 1,
    },
  ];

  return completions;
});

connection.onCompletionResolve((item) => {
  if (item.data === 1) {
    item.detail = "An {{if ...}} directive";
    item.documentation = "Some more documentation";
  }

  return item;
});

connection.listen();
