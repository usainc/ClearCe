import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";
import { AppProvider } from "./stores/AppContext";
import { ProcessingProvider } from "./stores/ProcessingContext";
import { QueueProvider } from "./stores/QueueContext";
import "./styles.css";
createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <AppProvider>
      <ProcessingProvider>
        <QueueProvider>
          <App />
        </QueueProvider>
      </ProcessingProvider>
    </AppProvider>
  </StrictMode>,
);
