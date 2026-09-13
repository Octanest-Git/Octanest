import { createRoot } from "octane";
import { App } from "./App.tsrx";

const el = document.getElementById("root");
if (el) {
  createRoot(el).render(App, { title: "Hello Octane" });
}
