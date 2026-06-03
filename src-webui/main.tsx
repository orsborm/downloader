// WebUI 入口文件
// 复用桌面端组件，通过 JSON-RPC API 与后端通信

import React from "react";
import ReactDOM from "react-dom/client";
import { WebUIApp } from "./App";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <WebUIApp />
  </React.StrictMode>
);
