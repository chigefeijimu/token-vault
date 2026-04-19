import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWalletStore } from "../stores/walletStore";
import { CHAINS } from "../types/wallet";
import { CustomToken } from "../types/wallet";

interface TokenInfo {
  name: string;
  symbol: string;
  decimals: number;
}

// Generate a blockie-style color from address
function addressToColor(address: string): string {
  const addr = address.toLowerCase().replace("0x", "");
  let hash = 0;
  for (let i = 0; i < addr.length; i++) {
    hash = addr.charCodeAt(i) + ((hash << 5) - hash);
  }
  const hue = Math.abs(hash % 360);
  return `hsl(${hue}, 60%, 50%)`;
}

// Generate blockie-like avatar SVG
function BlockieAvatar({ address, size = 32 }: { address: string; size?: number }) {
  const addr = address.toLowerCase().replace("0x", "");
  const colors = [
    addressToColor(address),
    addressToColor(address + "00"),
    addressToColor(address + "000"),
  ];
  
  // Create a simple 4x4 pattern based on address
  const grid: number[] = [];
  for (let i = 0; i < 16; i++) {
    const char = addr.charCodeAt(i % addr.length);
    grid.push((char + i) % 2);
  }

  return (
    <svg width={size} height={size} viewBox="0 0 8 8" style={{ borderRadius: "50%" }}>
      {grid.map((val, idx) => {
        const x = idx % 4;
        const y = Math.floor(idx / 4);
        const colorIdx = (x + y) % 2;
        return (
          <rect
            key={idx}
            x={x}
            y={y}
            width="2"
            height="2"
            fill={val ? colors[colorIdx % colors.length] : colors[2]}
          />
        );
      })}
    </svg>
  );
}

export function TokenManagement() {
  const { wallets, selectedChain, setSelectedChain, customTokens, addCustomToken, removeCustomToken, getCustomTokens } = useWalletStore();
  
  const [contractAddress, setContractAddress] = useState("");
  const [chainId, setChainId] = useState(selectedChain);
  const [fetching, setFetching] = useState(false);
  const [fetchedToken, setFetchedToken] = useState<TokenInfo | null>(null);
  const [fetchError, setFetchError] = useState("");
  const [adding, setAdding] = useState(false);
  const [selectedWalletId, setSelectedWalletId] = useState("");
  const [tokenBalances, setTokenBalances] = useState<Record<string, string>>({});

  const selectedWallet = wallets.find((w) => w.id === selectedWalletId);
  const tokensOnChain = getCustomTokens(chainId);

  useEffect(() => {
    setChainId(selectedChain);
  }, [selectedChain]);

  useEffect(() => {
    // Fetch balances for tokens when wallet is selected
    const fetchTokensBalances = async () => {
      if (!selectedWallet) return;
      
      const newBalances: Record<string, string> = {};
      for (const token of tokensOnChain) {
        try {
          const balance = await invoke<{ balance: string }>("get_erc20_balance", {
            tokenAddress: token.address,
            walletAddress: selectedWallet.address,
            chainId: token.chainId,
          });
          // Format the balance
          const formatted = formatTokenBalance(balance.balance, token.decimals);
          newBalances[token.address.toLowerCase()] = formatted;
        } catch {
          newBalances[token.address.toLowerCase()] = "0";
        }
      }
      setTokenBalances(newBalances);
    };

    fetchTokensBalances();
  }, [selectedWallet, tokensOnChain, selectedChain]);

  function validateAddress(addr: string): boolean {
    return /^0x[0-9a-fA-F]{40}$/.test(addr);
  }

  async function handleFetchToken() {
    if (!validateAddress(contractAddress)) {
      setFetchError("Invalid contract address format");
      setFetchedToken(null);
      return;
    }

    // Check for duplicates
    const exists = customTokens.some(
      t => t.address.toLowerCase() === contractAddress.toLowerCase() && t.chainId === chainId
    );
    if (exists) {
      setFetchError("Token already added on this chain");
      setFetchedToken(null);
      return;
    }

    setFetching(true);
    setFetchError("");
    setFetchedToken(null);

    try {
      const info = await invoke<TokenInfo>("get_token_info", {
        tokenAddress: contractAddress,
        chainId,
      });
      setFetchedToken(info);
    } catch (e) {
      setFetchError(`Failed to fetch token: ${String(e)}`);
      setFetchedToken(null);
    } finally {
      setFetching(false);
    }
  }

  async function handleAddToken() {
    if (!fetchedToken) return;
    setAdding(true);

    const newToken: CustomToken = {
      address: contractAddress.toLowerCase(),
      name: fetchedToken.name,
      symbol: fetchedToken.symbol,
      decimals: fetchedToken.decimals,
      chainId,
    };

    addCustomToken(newToken);
    
    // Reset form
    setContractAddress("");
    setFetchedToken(null);
    setFetchError("");
    setAdding(false);
  }

  function handleRemoveToken(address: string, tokenChainId: number) {
    removeCustomToken(address, tokenChainId);
  }

  function formatTokenBalance(balance: string, decimals: number): string {
    if (!balance || balance === "0") return "0";
    
    const bal = BigInt(balance);
    const divisor = BigInt(10 ** decimals);
    
    const whole = bal / divisor;
    const fraction = bal % divisor;
    
    const fractionStr = fraction.toString().padStart(decimals, "0").replace(/0+$/, "");
    
    if (fractionStr === "") return whole.toString();
    return `${whole}.${fractionStr}`;
  }

  return (
    <div className="p-4 sm:p-6 max-w-xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-vault-text">Token Management</h1>
        <p className="text-sm text-gray-500 mt-0.5">Add and manage custom ERC20 tokens</p>
      </div>

      {/* Add Token Form */}
      <div className="bg-vault-card border border-vault-border rounded-xl p-4 space-y-4">
        <h2 className="text-lg font-semibold text-vault-text">Add Custom Token</h2>

        {/* Chain selector */}
        <div className="space-y-1">
          <label className="text-sm text-gray-400">Network</label>
          <select
            value={chainId}
            onChange={(e) => {
              setChainId(Number(e.target.value));
              setSelectedChain(Number(e.target.value));
              setFetchedToken(null);
              setFetchError("");
            }}
            className="w-full bg-vault-bg border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm"
          >
            {CHAINS.map((c) => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </select>
        </div>

        {/* Contract Address */}
        <div className="space-y-1">
          <label className="text-sm text-gray-400">Token Contract Address</label>
          <input
            type="text"
            value={contractAddress}
            onChange={(e) => {
              setContractAddress(e.target.value);
              setFetchedToken(null);
              setFetchError("");
            }}
            placeholder="0x..."
            className="w-full bg-vault-bg border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm font-mono"
          />
        </div>

        {fetchError && (
          <div className="bg-red-900/30 border border-red-700 rounded-lg px-3 py-2 text-red-400 text-sm">
            {fetchError}
          </div>
        )}

        {/* Token Preview */}
        {fetchedToken && (
          <div className="bg-vault-bg border border-vault-border rounded-lg p-3 space-y-2">
            <div className="flex items-center gap-3">
              <BlockieAvatar address={contractAddress} size={40} />
              <div>
                <div className="font-semibold text-vault-text">{fetchedToken.name}</div>
                <div className="text-sm text-gray-400">{fetchedToken.symbol}</div>
              </div>
            </div>
            <div className="text-xs text-gray-500">
              Decimals: {fetchedToken.decimals}
            </div>
            <div className="text-xs text-gray-500 font-mono break-all">
              {contractAddress}
            </div>
          </div>
        )}

        <div className="flex gap-3">
          <button
            onClick={handleFetchToken}
            disabled={!contractAddress || fetching}
            className="flex-1 py-2.5 bg-vault-card border border-vault-border text-vault-text rounded-lg font-medium text-sm disabled:opacity-40 hover:bg-vault-bg transition-colors"
          >
            {fetching ? "Fetching..." : "Fetch Token Info"}
          </button>
          {fetchedToken && (
            <button
              onClick={handleAddToken}
              disabled={adding}
              className="flex-1 py-2.5 bg-vault-gradient text-white rounded-lg font-medium text-sm disabled:opacity-40"
            >
              {adding ? "Adding..." : "Add Token"}
            </button>
          )}
        </div>
      </div>

      {/* Token List */}
      <div className="space-y-4">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-semibold text-vault-text">My Tokens</h2>
          <select
            value={chainId}
            onChange={(e) => setChainId(Number(e.target.value))}
            className="bg-vault-card border border-vault-border rounded-lg px-3 py-1.5 text-vault-text text-sm"
          >
            {CHAINS.map((c) => (
              <option key={c.id} value={c.id}>{c.name}</option>
            ))}
          </select>
        </div>

        {tokensOnChain.length === 0 ? (
          <div className="bg-vault-card border border-vault-border rounded-xl p-6 text-center">
            <div className="text-3xl mb-2">📝</div>
            <div className="text-gray-400 text-sm">No custom tokens added on this network</div>
          </div>
        ) : (
          <div className="space-y-2">
            {tokensOnChain.map((token) => (
              <div
                key={`${token.chainId}-${token.address}`}
                className="bg-vault-card border border-vault-border rounded-xl p-3 flex items-center justify-between"
              >
                <div className="flex items-center gap-3">
                  <BlockieAvatar address={token.address} size={40} />
                  <div>
                    <div className="font-semibold text-vault-text">{token.name}</div>
                    <div className="flex items-center gap-2 text-sm">
                      <span className="text-gray-400">{token.symbol}</span>
                      <span className="text-gray-600">•</span>
                      <span className="text-xs text-gray-500">
                        {CHAINS.find(c => c.id === token.chainId)?.name}
                      </span>
                    </div>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <div className="text-right">
                    <div className="text-sm font-mono text-vault-text">
                      {tokenBalances[token.address.toLowerCase()] || "—"}
                    </div>
                    <div className="text-xs text-gray-500">balance</div>
                  </div>
                  <button
                    onClick={() => handleRemoveToken(token.address, token.chainId)}
                    className="p-2 text-red-400 hover:text-red-300 hover:bg-red-900/30 rounded-lg transition-colors"
                    title="Remove token"
                  >
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Wallet selector for balance display */}
      {tokensOnChain.length > 0 && (
        <div className="bg-vault-card border border-vault-border rounded-xl p-4">
          <div className="space-y-1">
            <label className="text-sm text-gray-400">Select Wallet to View Balances</label>
            <select
              value={selectedWalletId}
              onChange={(e) => setSelectedWalletId(e.target.value)}
              className="w-full bg-vault-bg border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm"
            >
              <option value="">Select wallet</option>
              {wallets.map((w) => (
                <option key={w.id} value={w.id}>
                  {w.name} ({w.address.slice(0,6)}...{w.address.slice(-4)})
                </option>
              ))}
            </select>
          </div>
        </div>
      )}
    </div>
  );
}
