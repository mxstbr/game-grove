import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { Store } from "@tauri-apps/plugin-store";
import "./App.css";

interface GameEntry {
  name: string;
  path: string;
  last_modified: number; // Unix timestamp
}

function App() {
  const [games, setGames] = useState<GameEntry[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedPath, setSelectedPath] = useState<string | null>(null);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [newGameName, setNewGameName] = useState("");
  const [creating, setCreating] = useState(false);
  const [gameType, setGameType] = useState<"2d" | "3d" | null>(null);
  const [createStep, setCreateStep] = useState<"type" | "name">("type");
  const [store, setStore] = useState<Store | null>(null);
  const [initializing, setInitializing] = useState(true);

  // Initialize store and load saved path on mount
  useEffect(() => {
    async function initializeStore() {
      try {
        const appStore = await Store.load('app_settings.json');
        setStore(appStore);
        
        // Load saved path from store
        const savedPath = await appStore.get<string>('selected_games_path');
        if (savedPath) {
          setSelectedPath(savedPath);
        }
      } catch (err) {
        console.error('Failed to initialize store:', err);
      } finally {
        setInitializing(false);
      }
    }
    
    initializeStore();
  }, []);

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
        // Sort by last modified (newest first)
        const sortedGames = result.sort((a, b) => b.last_modified - a.last_modified);
        setGames(sortedGames);
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

  // Save selected path to store whenever it changes
  useEffect(() => {
    if (!store || initializing) return;
    
    async function saveSelectedPath() {
      if (!store) return; // Extra check to satisfy TypeScript
      try {
        await store.set('selected_games_path', selectedPath);
        await store.save(); // Ensure changes are persisted to disk
      } catch (err) {
        console.error('Failed to save selected path:', err);
      }
    }
    
    saveSelectedPath();
  }, [selectedPath, store, initializing]);

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
    if (!selectedPath || !newGameName.trim() || !gameType) return;

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
        folderName: formattedName,
        gameType: gameType
      });

      // Reload games list
      const result = await invoke<GameEntry[]>("read_folders_from_path", { 
        folderPath: selectedPath 
      });
      // Sort by last modified (newest first)
      const sortedGames = result.sort((a, b) => b.last_modified - a.last_modified);
      setGames(sortedGames);
      
      // Reset modal state
      setShowCreateModal(false);
      setNewGameName("");
      setGameType(null);
      setCreateStep("type");
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
    setGameType(null);
    setCreateStep("type");
    setError(null);
  }

  function handleCancelCreate() {
    setShowCreateModal(false);
    setNewGameName("");
    setGameType(null);
    setCreateStep("type");
    setError(null);
  }

  async function openInCursor(gamePath: string) {
    try {
      await invoke("open_in_cursor", { folderPath: gamePath });
    } catch (err) {
      console.error("Error opening in Cursor:", err);
      setError(err instanceof Error ? err.message : "Failed to open in Cursor");
    }
  }

  async function openInBrowser(gamePath: string) {
    try {
      await invoke("open_html_in_browser", { folderPath: gamePath });
    } catch (err) {
      console.error("Error opening in browser:", err);
      setError(err instanceof Error ? err.message : "Failed to open in browser");
    }
  }

  // Show loading state while initializing
  if (initializing) {
    return (
      <main className="container">
        <div className="loading">
          <span className="loading-icon">⏳</span>
          <p>Loading Game Grove...</p>
        </div>
      </main>
    );
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
                      <div className="game-actions">
                        <button
                          className="action-button cursor-button"
                          onClick={() => openInCursor(game.path)}
                          title="Open in Cursor"
                        >
                          <span className="button-icon">💻</span>
                          Cursor
                        </button>
                        <button
                          className="action-button browser-button"
                          onClick={() => openInBrowser(game.path)}
                          title="Open in Browser"
                        >
                          <span className="button-icon">🌐</span>
                          Browser
                        </button>
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
            {createStep === "type" ? (
              <>
                <h2>🎮 Choose Game Type</h2>
                <p className="modal-description">What kind of game do you want to create?</p>
                <div className="game-type-selection">
                  <button
                    className={`game-type-button ${gameType === "2d" ? "selected" : ""}`}
                    onClick={() => {
                      setGameType("2d");
                      setCreateStep("name");
                    }}
                  >
                    <div className="game-type-icon">🎲</div>
                    <div className="game-type-title">2D Game</div>
                    <div className="game-type-description">Perfect for side-scrollers, puzzle games, and classic arcade fun!</div>
                  </button>
                  <button
                    className={`game-type-button ${gameType === "3d" ? "selected" : ""}`}
                    onClick={() => {
                      setGameType("3d");
                      setCreateStep("name");
                    }}
                  >
                    <div className="game-type-icon">🎯</div>
                    <div className="game-type-title">3D Game</div>
                    <div className="game-type-description">Great for adventures, exploration, and immersive experiences!</div>
                  </button>
                </div>
                <div className="modal-buttons">
                  <button
                    onClick={handleCancelCreate}
                    className="cancel-button"
                  >
                    Cancel
                  </button>
                </div>
              </>
            ) : (
              <>
                <h2>🚀 Name Your Game</h2>
                <p className="modal-description">
                  Creating a <strong>{gameType?.toUpperCase()} game</strong> - What should we call it?
                </p>
                <input
                  type="text"
                  placeholder="Enter game name"
                  value={newGameName}
                  onChange={(e) => setNewGameName(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && !creating && newGameName.trim()) {
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
                    onClick={() => setCreateStep("type")}
                    disabled={creating}
                    className="back-button"
                  >
                    ← Back
                  </button>
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
                    {creating ? 'Creating...' : 'Create Game'}
                  </button>
                </div>
              </>
            )}
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