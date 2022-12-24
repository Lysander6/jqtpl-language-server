#!/usr/bin/env node

import {
  CompletionItem,
  CompletionItemKind,
  createConnection,
  InitializeResult,
  InsertTextFormat,
  ProposedFeatures,
  TextDocumentPositionParams,
  TextDocumentSyncKind,
  TextDocuments,
} from "vscode-languageserver/node";

import { TextDocument } from "vscode-languageserver-textdocument";

const SNIPPETS = [
  { label: "{{= ...}}", newText: "{{= $1}}$0" },
  { label: "{{if ...}}", newText: "{{if $1}}$2{{/if}}$0" },
  { label: "{{var ...}}", newText: "{{var ${1:locals.}$2 = $3}}$0" },
  { label: "{{html ...}}", newText: "{{html $1}}$0" },
  { label: "{{each ...}}", newText: "{{each({${2:i}, ${3:item}}) $1}}$0" },
  {
    label: "{{tmpl ...}}",
    newText: "{{tmpl({$3}) partials.getTemplate($1, $2)}}$0",
  },
];

const connection = createConnection(ProposedFeatures.all);
const documents = new TextDocuments(TextDocument);

connection.onInitialize((_params) => {
  const result: InitializeResult = {
    capabilities: {
      completionProvider: {
        triggerCharacters: ["{"],
      },
      textDocumentSync: TextDocumentSyncKind.Incremental,
      workspace: {
        workspaceFolders: {
          supported: true,
        },
      },
    },
  };

  return result;
});

connection.onCompletion((params: TextDocumentPositionParams) => {
  const doc = documents.get(params.textDocument.uri);

  if (!doc) {
    return [];
  }

  // 4 characters wide slice surrounding the cursor
  const surroundingText = doc.getText({
    start: {
      line: params.position.line,
      character: params.position.character - 2,
    },
    end: {
      line: params.position.line,
      character: params.position.character + 2,
    },
  });
  const editRange = {
    start: { ...params.position },
    end: { ...params.position },
  };

  // expand edit range to replace any curly brackets that were already inserted
  // by the user
  if (surroundingText[0] == "{") {
    editRange.start.character -= 1;
  }
  if (surroundingText[1] == "{") {
    editRange.start.character -= 1;
  }
  if (surroundingText[2] == "}") {
    editRange.end.character += 1;
  }
  if (surroundingText[3] == "}") {
    editRange.end.character += 1;
  }

  const completions: CompletionItem[] = SNIPPETS.map(({ label, newText }) => ({
    label,
    kind: CompletionItemKind.Snippet,
    textEdit: {
      newText,
      range: editRange,
    },
    insertTextFormat: InsertTextFormat.Snippet,
  }));

  return completions;
});

documents.listen(connection);

connection.listen();
