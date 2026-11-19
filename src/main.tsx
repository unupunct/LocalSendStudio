import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter } from "react-router-dom";
import App from "./App";
import { AppProvider } from "./lib/store";
import "./styles.css";

try {
  const t = localStorage.getItem("theme");
  if (t === "light" || t === "dark") document.documentElement.setAttribute("data-theme", t);
} catch {}

async function start() {
  // Screenshot/dev mode only: `VITE_DEMO=1 npm run dev`. Removed from production builds.
  if (import.meta.env.VITE_DEMO) {
    await (await import("./demo")).installDemo();
  }
  ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
      <HashRouter>
        <AppProvider>
          <App />
        </AppProvider>
      </HashRouter>
    </React.StrictMode>
  );
}
start();
