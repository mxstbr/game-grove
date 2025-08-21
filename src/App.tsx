import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import "./App.css";

interface GameEntry {
  name: string;
  path: string;
}

function App() {
  const [games, setGames] = useState<GameEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);

  // Load games when selected path changes
  useEffect(() => {
    if (!selectedPath) return;
    
    async function loadGames() {
      try {
        setLoading(true);
        setError(null);
        const result = await invoke<GameEntry[]>("read_folders_from_path", { 
          folderPath: selectedPath 
        });
        setGames(result);
      } catch (err) {
        console.error(err);
        setError(err instanceof Error ? err.message : "Failed to load games");
        setGames([]);
      } finally {
        setLoading(false);
      }
    }

    loadGames();
  }, [selectedPath]);

  async function selectGamesFolder() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select your games folder"
      });
      
      if (selected && typeof selected === 'string') {
        setSelectedPath(selected);
      }
    } catch (err) {
      console.error("Error selecting games folder:", err);
      setError("Failed to select games folder");
    }
  }

  return (
    <main className="container">
      {selectedPath && (
        <button 
          onClick={selectGamesFolder}
          className="settings-button"
          title="Change games folder"
        >
          ⚙️
        </button>
      )}
      
      <div className="header">
        <h1 className="title">
          <span className="game-icon">🎮</span>
          Game Grove
          <span className="game-icon">🎯</span>
        </h1>
        <p className="subtitle">Your awesome game collection!</p>
      </div>

      <div className="games-section">
        {!selectedPath ? (
          <div className="welcome-section">
            <div className="welcome-message">
              <h2>🚀 Ready to play?</h2>
              <p>Choose where your games live!</p>
            </div>
            <button 
              onClick={selectGamesFolder}
              className="select-button"
            >
              <span className="button-icon">📂</span>
              Choose Games Folder
            </button>
          </div>
        ) : (
          <>

            {loading && (
              <div className="loading">
                <span className="loading-icon">⏳</span>
                <p>Loading your games...</p>
              </div>
            )}
            
            {error && (
              <div className="error">
                <span className="error-icon">😕</span>
                <p>Oops! {error}</p>
              </div>
            )}
            
            {!loading && !error && games.length === 0 && (
              <div className="no-games">
                <span className="no-games-icon">📭</span>
                <p>No games found here yet!</p>
                <p className="hint">Make sure each game has its own folder in your games directory.</p>
              </div>
            )}
            
            {!loading && !error && games.length > 0 && (
              <div className="games-container">
                <div className="games-info">
                  <h2 className="games-title">
                    <span>🎲</span> Your Games ({games.length})
                  </h2>
                  <div className="games-location">
                    📂 {selectedPath}
                  </div>
                </div>
                <div className="games-grid">
                  {games.map((game, index) => (
                    <div 
                      key={game.path} 
                      className="game-card"
                      style={{
                        animationDelay: `${index * 0.05}s`
                      }}
                    >
                      <div className="game-icon-large">
                        {getGameIcon(index)}
                      </div>
                      <div className="game-name">
                        {game.name}
                      </div>
                      <div className="game-path">
                        {game.path}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </>
        )}
      </div>
    </main>
  );
}

function getGameIcon(index: number): string {
  const icons = ["🎮", "🎯", "🎲", "🎨", "🏆", "⚡", "🌟", "🚀", "🎪", "🎸", "🏰", "🦄", "🐉", "⚔️", "🛡️", "💎"];
  return icons[index % icons.length];
}

export default App;