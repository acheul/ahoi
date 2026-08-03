import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { App } from "./App";

// StrictMode is deliberate: it double-mounts and discards renders, which is
// exactly what the adapter's grace-timer cleanup must survive
createRoot(document.getElementById("root")!).render(
    <StrictMode>
        <App />
    </StrictMode>,
);
