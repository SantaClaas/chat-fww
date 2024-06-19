/* @refresh reload */
import { render } from "solid-js/web";

import "./index.css";
import { Route, Router } from "@solidjs/router";
import Index from "./routes/Index";
import { ContextProvider } from "./context";
import SetUp from "./routes/SetUp";
import Chat from "./routes/Chat";

render(
  () => (
    <ContextProvider>
      <Router>
        <Route path="/" component={Index} />
        <Route path="/setup" component={SetUp} />
        <Route path="/chats/:name" component={Chat} />
      </Router>
    </ContextProvider>
  ),
  document.body
);
