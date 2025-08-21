import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

interface FolderEntry {
  name: string;
  path: string;
}

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [folders, setFolders] = useState<FolderEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  useEffect(() => {
    // Load folders from ~/src when component mounts
    async function loadFolders() {
      try {
        setLoading(true);
        console.log(invoke)
        const result = await invoke<FolderEntry[]>("read_src_folders");
        setFolders(result);
        setError(null);
      } catch (err) {
        console.error(err)
        setError(err instanceof Error ? err.message : "Failed to load folders");
        setFolders([]);
      } finally {
        setLoading(false);
      }
    }

    loadFolders();
  }, []);

  return (
    <main className="container">
      <h1>Welcome to Tauri + React</h1>

      <div className="row">
        <a href="https://vite.dev" target="_blank">
          <img src="/vite.svg" className="logo vite" alt="Vite logo" />
        </a>
        <a href="https://tauri.app" target="_blank">
          <img src="/tauri.svg" className="logo tauri" alt="Tauri logo" />
        </a>
        <a href="https://react.dev" target="_blank">
          <img src={reactLogo} className="logo react" alt="React logo" />
        </a>
      </div>
      <p>Click on the Tauri, Vite, and React logos to learn more.</p>

      <form
        className="row"
        onSubmit={(e) => {
          e.preventDefault();
          greet();
        }}
      >
        <input
          id="greet-input"
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="Enter a name..."
        />
        <button type="submit">Greet</button>
      </form>
      <p>{greetMsg}</p>

      <div style={{ marginTop: "2rem", textAlign: "left" }}>
        <h2>Folders in ~/src directory:</h2>
        {loading && <p>Loading folders...</p>}
        {error && <p style={{ color: "red" }}>Error: {error}</p>}
        {!loading && !error && folders.length === 0 && (
          <p>No folders found in ~/src directory</p>
        )}
        {!loading && !error && folders.length > 0 && (
          <ul style={{ 
            listStyle: "none", 
            maxHeight: "400px",
            overflowY: "auto",
            border: "1px solid #333",
            borderRadius: "8px",
            padding: "1rem"
          }}>
            {folders.map((folder) => (
              <li 
                key={folder.path} 
                style={{ 
                  padding: "0.5rem",
                  borderBottom: "1px solid #222",
                  marginBottom: "0.5rem"
                }}
              >
                <strong>📁 {folder.name}</strong>
                <div style={{ fontSize: "0.85em", color: "#888", marginTop: "0.25rem" }}>
                  {folder.path}
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    </main>
  );
}

export default App;
