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
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newGameName, setNewGameName] = useState("");
  const [creating, setCreating] = useState(false);

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

  // Convert game name to filesystem-friendly format
  function formatGameNameForFilesystem(name: string): string {
    return name
      .toLowerCase()
      .trim()
      .replace(/[^a-z0-9\s-]/g, '') // Remove special characters except spaces and hyphens
      .replace(/\s+/g, '-')         // Replace spaces with hyphens
      .replace(/-+/g, '-')          // Replace multiple hyphens with single hyphen
      .replace(/^-+|-+$/g, '');     // Remove leading/trailing hyphens
  }

  async function createNewGame() {
    if (!selectedPath || !newGameName.trim()) return;

    const formattedName = formatGameNameForFilesystem(newGameName);
    
    if (!formattedName) {
      setError("Please enter a valid game name");
      return;
    }

    try {
      setCreating(true);
      setError(null);
      
      await invoke("create_game_folder", {
        parentPath: selectedPath,
        folderName: formattedName
      });

      // Reload games list
      const result = await invoke<GameEntry[]>("read_folders_from_path", { 
        folderPath: selectedPath 
      });
      setGames(result);
      
      // Reset modal state
      setShowCreateModal(false);
      setNewGameName("");
    } catch (err) {
      console.error("Error creating game folder:", err);
      setError(err instanceof Error ? err.message : "Failed to create game folder");
    } finally {
      setCreating(false);
    }
  }

  function handleCreateGameClick() {
    setShowCreateModal(true);
    setNewGameName("");
    setError(null);
  }

  function handleCancelCreate() {
    setShowCreateModal(false);
    setNewGameName("");
    setError(null);
  }

  return (
    <main className="container">
      {selectedPath && (
        <>
          <button 
            onClick={selectGamesFolder}
            className="settings-button"
            title="Change games folder"
          >
            ⚙️
          </button>
          <button
            onClick={handleCreateGameClick}
            className="create-button"
            title="Create new game"
          >
            ➕ New Game
          </button>
        </>
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

      {/* Create Game Modal */}
      {showCreateModal && (
        <div className="modal-overlay" onClick={handleCancelCreate}>
          <div className="modal" onClick={(e) => e.stopPropagation()}>
            <h2>Create New Game</h2>
            <input
              type="text"
              placeholder="Enter game name"
              value={newGameName}
              onChange={(e) => setNewGameName(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !creating) {
                  createNewGame();
                } else if (e.key === 'Escape') {
                  handleCancelCreate();
                }
              }}
              autoFocus
              className="game-name-input"
            />
            {newGameName && (
              <p className="folder-preview">
                Folder name: <strong>{formatGameNameForFilesystem(newGameName) || '...'}</strong>
              </p>
            )}
            {error && (
              <p className="modal-error">{error}</p>
            )}
            <div className="modal-buttons">
              <button
                onClick={handleCancelCreate}
                disabled={creating}
                className="cancel-button"
              >
                Cancel
              </button>
              <button
                onClick={createNewGame}
                disabled={creating || !newGameName.trim()}
                className="confirm-button"
              >
                {creating ? 'Creating...' : 'Create'}
              </button>
            </div>
          </div>
        </div>
      )}
    </main>
  );
}

function getGameIcon(index: number): string {
  const icons = ["🎮", "🎯", "🎲", "🎨", "🏆", "⚡", "🌟", "🚀", "🎪", "🎸", "🏰", "🦄", "🐉", "⚔️", "🛡️", "💎"];
  return icons[index % icons.length];
}

export default App;