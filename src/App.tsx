import { useState, useEffect } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import "./App.css";

interface FolderEntry {
  name: string;
  path: string;
}

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");
  const [folders, setFolders] = useState<FolderEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  // Load folders when selected path changes
  useEffect(() => {
    if (!selectedPath) return;
    
    async function loadFolders() {
      try {
        setLoading(true);
        setError(null);
        const result = await invoke<FolderEntry[]>("read_folders_from_path", { 
          folderPath: selectedPath 
        });
        setFolders(result);
      } catch (err) {
        console.error(err);
        setError(err instanceof Error ? err.message : "Failed to load folders");
        setFolders([]);
      } finally {
        setLoading(false);
      }
    }

    loadFolders();
  }, [selectedPath]);

  async function selectFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select a folder to browse"
      });
      
      if (selected && typeof selected === 'string') {
        setSelectedPath(selected);
      }
    } catch (err) {
      console.error("Error selecting folder:", err);
      setError("Failed to select folder");
    }
  }

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
        <h2>Folder Browser</h2>
        
        <div style={{ marginBottom: "1rem" }}>
          <button 
            onClick={selectFolder}
            style={{
              padding: "0.75rem 1.5rem",
              fontSize: "1rem",
              borderRadius: "8px",
              border: "1px solid #646cff",
              backgroundColor: "#1a1a1a",
              color: "#fff",
              cursor: "pointer",
              transition: "all 0.25s"
            }}
          >
            📁 Select Folder
          </button>
          
          {selectedPath && (
            <div style={{ 
              marginTop: "0.5rem", 
              padding: "0.5rem",
              backgroundColor: "#2a2a2a",
              borderRadius: "4px",
              fontSize: "0.9em"
            }}>
              <strong>Selected path:</strong> {selectedPath}
            </div>
          )}
        </div>

        {!selectedPath && (
          <p style={{ color: "#888" }}>Please select a folder to browse its contents</p>
        )}
        
        {selectedPath && loading && <p>Loading folders...</p>}
        {selectedPath && error && <p style={{ color: "red" }}>Error: {error}</p>}
        {selectedPath && !loading && !error && folders.length === 0 && (
          <p>No folders found in the selected directory</p>
        )}
        {selectedPath && !loading && !error && folders.length > 0 && (
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
