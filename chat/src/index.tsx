/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";
import { Route, Router } from "@solidjs/router";
import Index from "./routes/Index";
import { ContextProvider } from "./context";
import SetUp from "./routes/SetUp";

render(
  () => (
    <ContextProvider>
      <Router>
        <Route path="/" component={Index} />
        <Route path="/setup" component={SetUp} />
      </Router>
    </ContextProvider>
  ),
  document.body
);
