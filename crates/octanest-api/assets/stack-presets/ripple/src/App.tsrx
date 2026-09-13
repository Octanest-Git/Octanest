type AppProps = {
  title: string;
};

export function App(props: AppProps) @{
  <main>
    <h1>{props.title as string}</h1>
    <p>Edit src/App.tsrx and save to reload.</p>
  </main>
}
