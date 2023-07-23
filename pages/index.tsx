import React, { useRef } from "react";
import ReactDOM from "react-dom";
// import Editor from "@monaco-editor/react";

function Ted() {
  const editorRef = useRef(null);

  return <canvas ref={editorRef} height={500} width={1400}></canvas>;
}

export default function Index() {
  const editorRef = useRef(null);

  function handleEditorDidMount(editor, monaco) {
    editorRef.current = editor;
  }

  function showValue() {
    alert(editorRef.current.getValue());
  }

  return (
    <div
      style={{
        width: "100vw",
        height: "100vh",
        backgroundImage: "url('./bg-image.jpg')",
      }}
    >
      <button onClick={showValue}>Show value</button>
      <Ted />
    </div>
  );
}
