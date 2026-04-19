import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import { LockScreen } from "./components/LockScreen";
import { useAutoLock } from "./hooks/useAutoLock";
import "./styles.css";

function AutoLockProvider({ children }: { children: React.ReactNode }) {
  useAutoLock();
  return <>{children}</>;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <AutoLockProvider>
        <App />
        <LockScreen />
      </AutoLockProvider>
    </BrowserRouter>
  </React.StrictMode>
);
