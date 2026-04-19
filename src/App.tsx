import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Chain, CHAINS } from "./types/wallet";

function App() {
  const [selectedChain, setSelectedChain] = useState<Chain>(CHAINS[0]);
  const [balance, setBalance] = useState<string>("");
  const [address, setAddress] = useState<string>("");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // Initialize app
  }, []);

  const handleCreateWallet = async () => {
    try {
      setLoading(true);
      const result = await invoke<{ address: string }>("create_wallet", {
        chainId: selectedChain.id,
      });
      setAddress(result.address);
    } catch (error) {
      console.error("Failed to create wallet:", error);
    } finally {
      setLoading(false);
    }
  };

  const handleGetBalance = async () => {
    if (!address) return;
    try {
      setLoading(true);
      const result = await invoke<{ balance: string }>("get_balance", {
        address,
        chainId: selectedChain.id,
      });
      setBalance(result.balance);
    } catch (error) {
      console.error("Failed to get balance:", error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="container">
      <header>
        <h1>Token Vault</h1>
        <p>Multi-chain Wallet Manager</p>
      </header>

      <div className="card">
        <h2>Select Chain</h2>
        <select
          value={selectedChain.id}
          onChange={(e) => {
            const chain = CHAINS.find((c) => c.id === Number(e.target.value));
            if (chain) setSelectedChain(chain);
          }}
        >
          {CHAINS.map((chain) => (
            <option key={chain.id} value={chain.id}>
              {chain.name} ({chain.symbol})
            </option>
          ))}
        </select>
      </div>

      <div className="card">
        <h2>Wallet</h2>
        <button onClick={handleCreateWallet} disabled={loading}>
          {loading ? "Creating..." : "Create New Wallet"}
        </button>

        {address && (
          <div className="wallet-info">
            <p>
              <strong>Address:</strong>
            </p>
            <code>{address}</code>
            <button onClick={handleGetBalance} disabled={loading}>
              Get Balance
            </button>
            {balance && <p>Balance: {balance} {selectedChain.symbol}</p>}
          </div>
        )}
      </div>

      <div className="card">
        <h2>Connected Chain</h2>
        <p>
          {selectedChain.name} - RPC: {selectedChain.rpcUrl}
        </p>
        <p>
          Explorer: <a href={selectedChain.explorerUrl} target="_blank" rel="noopener noreferrer">{selectedChain.explorerUrl}</a>
        </p>
      </div>
    </div>
  );
}

export default App;