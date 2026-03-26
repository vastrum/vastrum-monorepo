import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "../wasm/pkg/vastrum_wasm";
import App from "./App.tsx";
import "./tailwind.css";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <App />
  </StrictMode>
);
