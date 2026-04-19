import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter } from "react-router-dom";
import App from "./App";
import { LockScreen } from "./components/LockScreen";
import { useAutoLock } from "./hooks/useAutoLock";
import { I18nProvider } from "./i18n/I18nProvider";
import "./styles.css";

function AutoLockProvider({ children }: { children: React.ReactNode }) {
  useAutoLock();
  return <>{children}</>;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <BrowserRouter>
      <I18nProvider>
        <AutoLockProvider>
          <App />
          <LockScreen />
        </AutoLockProvider>
      </I18nProvider>
    </BrowserRouter>
  </React.StrictMode>
);
