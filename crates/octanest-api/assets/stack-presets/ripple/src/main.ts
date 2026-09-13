import { mount } from "ripple";
import { App } from "./App.tsrx";

const target = document.getElementById("root");
if (target) {
  mount(App, {
    props: { title: "Hello Ripple" },
    target,
  });
}
