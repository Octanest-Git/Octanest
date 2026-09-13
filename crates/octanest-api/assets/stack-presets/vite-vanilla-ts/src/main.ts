import "./style.css";
const app = document.querySelector<HTMLDivElement>("#app");
if (app) {
  app.innerHTML = `
    <h1>Hello Vite</h1>
    <p>Vanilla TypeScript — edit <code>src/main.ts</code>.</p>
  `;
}
