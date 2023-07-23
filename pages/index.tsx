import React, { useRef } from "react";
import ReactDOM from "react-dom";
// import Editor from "@monaco-editor/react";

export default function Index() {
  const editorRef = useRef(null);

  function handleEditorDidMount(editor, monaco) {
    editorRef.current = editor;
  }

  function showValue() {
    alert(editorRef.current.getValue());
  }

  return (
    <div>
      <button onClick={showValue}>Show value</button>
      {/*
      <Editor
        height="90vh"
        defaultLanguage="javascript"
        defaultValue="// some comment"
        onMount={handleEditorDidMount}
      />
      */}
    </div>
  );
}
