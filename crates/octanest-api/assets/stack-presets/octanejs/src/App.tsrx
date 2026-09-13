import { useState } from "octane";

type AppProps = {
  title: string;
};

export function App(props: AppProps) @{
  const [count, setCount] = useState(0);

  <main>
    <h1>{props.title as string}</h1>
    <p>{"Button pressed " + count + " times"}</p>
    <button onClick={() => setCount((c) => c + 1)}>Press me</button>
  </main>
}
