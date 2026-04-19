import { useEffect, useState } from "react";
import { useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { useWalletStore } from "../stores/walletStore";
import type { WalletInfo } from "../types/wallet";

export default function WalletDashboard() {
  const navigate = useNavigate();
  const { 
    wallets, 
    activeWallet, 
    setActiveWallet, 
    isLoading,
    setIsLoading,
    activeChain,
    setActiveChain,
    chains,
    balances,
    fetchBalance,
  } = useWalletStore();

  const [fetchingBalances, setFetchingBalances] = useState<Set<string>>(new Set());

  // Load wallets from backend on mount
  useEffect(() => {
    loadWallets();
  }, []);

  // Fetch balances when wallets or active chain changes
  useEffect(() => {
    if (wallets.length > 0 && activeChain) {
      fetchBalancesForWallets();
    }
  }, [wallets, activeChain]);

  async function loadWallets() {
    setIsLoading(true);
    try {
      const result = await invoke<WalletInfo[]>("get_wallets");
      const store = useWalletStore.getState();
      store.wallets = result.map(w => ({
        id: w.id,
        name: w.name,
        address: w.address,
        isActive: false,
        createdAt: w.created_at,
      }));
    } catch (err) {
      console.error("Failed to load wallets:", err);
    } finally {
      setIsLoading(false);
    }
  }

  async function fetchBalancesForWallets() {
    for (const wallet of wallets) {
      const balanceKey = `${wallet.address}-${activeChain.id}`;
      if (!balances.has(balanceKey) && !fetchingBalances.has(balanceKey)) {
        setFetchingBalances(prev => new Set(prev).add(balanceKey));
        try {
          await fetchBalance(wallet.address, activeChain.id);
        } catch (err) {
          console.error(`Failed to fetch balance for ${wallet.address}:`, err);
        } finally {
          setFetchingBalances(prev => {
            const next = new Set(prev);
            next.delete(balanceKey);
            return next;
          });
        }
      }
    }
  }

  function getBalanceDisplay(walletAddress: string): string {
    const balanceKey = `${walletAddress}-${activeChain.id}`;
    const balance = balances.get(balanceKey);
    if (!balance) return "—";
    return `${balance.balance_formatted} ${balance.symbol}`;
  }

  function isFetchingBalance(walletAddress: string): boolean {
    const balanceKey = `${walletAddress}-${activeChain.id}`;
    return fetchingBalances.has(balanceKey) || !balances.has(balanceKey);
  }

  function handleSelectWallet(walletId: string) {
    setActiveWallet(walletId);
  }

  function handleViewWallet(walletId: string) {
    navigate(`/wallet/${walletId}`);
  }

  function handleChainChange(chainId: number) {
    const chain = chains.find(c => c.id === chainId);
    if (chain) {
      setActiveChain(chain);
    }
  }

  return (
    <div className="space-y-6 pb-20">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">My Wallets</h2>
        <button
          onClick={() => navigate("/create")}
          className="px-4 py-2 bg-vault-gradient text-white rounded-lg font-medium hover:opacity-90 transition-opacity"
        >
          + New Wallet
        </button>
      </div>

      {/* Chain Selector */}
      <div className="flex items-center gap-2 overflow-x-auto pb-2">
        <span className="text-sm text-vault-text-secondary shrink-0">Chain:</span>
        {chains.map((chain) => (
          <button
            key={chain.id}
            onClick={() => handleChainChange(chain.id)}
            className={`px-3 py-1.5 text-sm rounded-lg shrink-0 transition-colors ${
              activeChain.id === chain.id
                ? "bg-vault-gradient text-white"
                : "bg-vault-card border border-vault-border text-vault-text-secondary hover:text-vault-text"
            }`}
          >
            {chain.name} ({chain.symbol})
          </button>
        ))}
      </div>

      {isLoading ? (
        <div className="text-center py-12 text-vault-text-secondary">
          Loading...
        </div>
      ) : wallets.length === 0 ? (
        <div className="text-center py-12">
          <div className="text-6xl mb-4">📦</div>
          <p className="text-vault-text-secondary mb-4">No wallets yet</p>
          <button
            onClick={() => navigate("/create")}
            className="px-6 py-3 bg-vault-gradient text-white rounded-lg font-medium hover:opacity-90 transition-opacity"
          >
            Create Your First Wallet
          </button>
        </div>
      ) : (
        <div className="grid gap-4">
          {wallets.map((wallet) => (
            <div
              key={wallet.id}
              className={`bg-vault-card border rounded-xl p-4 cursor-pointer transition-all hover:border-vault-gradient ${
                activeWallet?.id === wallet.id ? "border-vault-gradient" : "border-vault-border"
              }`}
              onClick={() => handleViewWallet(wallet.id)}
            >
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <div className="flex items-center gap-2 mb-1">
                    <h3 className="font-semibold text-lg">{wallet.name}</h3>
                    {activeWallet?.id === wallet.id && (
                      <span className="px-2 py-0.5 text-xs bg-vault-success/20 text-vault-success rounded">
                        Active
                      </span>
                    )}
                  </div>
                  <div className="font-mono text-sm text-vault-text-secondary break-all">
                    {wallet.address}
                  </div>
                  <div className="flex items-center gap-4 mt-2">
                    <div className="text-xs text-vault-text-secondary">
                      Created: {new Date(wallet.createdAt).toLocaleDateString()}
                    </div>
                    <div className="text-xs font-medium">
                      {isFetchingBalance(wallet.address) ? (
                        <span className="text-vault-text-secondary">Loading balance...</span>
                      ) : (
                        <span className="text-vault-gradient">
                          {getBalanceDisplay(wallet.address)}
                        </span>
                      )}
                    </div>
                  </div>
                </div>
                <div className="flex flex-col gap-2">
                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      handleSelectWallet(wallet.id);
                    }}
                    className="px-3 py-1 text-sm border border-vault-border rounded hover:bg-vault-border transition-colors"
                  >
                    {activeWallet?.id === wallet.id ? "Selected" : "Select"}
                  </button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
