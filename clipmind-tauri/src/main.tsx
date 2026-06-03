import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./style.css";

try {
  const root = document.getElementById("root");
  if (!root) {
    document.body.innerHTML = '<div style="color:red;padding:20px;font-family:monospace">ERROR: #root not found</div>';
    throw new Error("Root element not found");
  }
  ReactDOM.createRoot(root).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>
  );
} catch (e) {
  console.error("[BuffBrain] Mount failed:", e);
  document.body.innerHTML = '<div style="color:red;padding:20px;font-family:monospace">MOUNT ERROR: ' + String(e) + '</div>';
}
