import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate } from "react-router-dom";
import { CHAINS } from "../types/wallet";

interface CreateWalletResponse {
  address: string;
  id: string;
  name: string;
}

export function CreateWallet() {
  const navigate = useNavigate();
  const [step, setStep] = useState<"form" | "created">("form");
  const [wallet, setWallet] = useState<CreateWalletResponse | null>(null);
  const [walletName, setWalletName] = useState("");
  const [chainId, setChainId] = useState(1);
  const [password, setPassword] = useState("");
  const [confirmPassword, setConfirmPassword] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleCreate() {
    if (!walletName.trim()) { setError("Wallet name is required"); return; }
    if (password.length < 8) { setError("Password must be at least 8 characters"); return; }
    if (password !== confirmPassword) { setError("Passwords do not match"); return; }
    setError("");
    setLoading(true);
    try {
      const result = await invoke<CreateWalletResponse>("create_wallet", {
        chainId,
        password,
        walletName,
      });
      setWallet(result);
      setStep("created");
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="p-4 sm:p-6 max-w-xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-vault-text">Create Wallet</h1>
        <p className="text-sm text-gray-500 mt-0.5">Generate a new multi-chain wallet</p>
      </div>

      {step === "form" && (
        <>
          <div className="space-y-1">
            <label className="text-sm text-gray-400">Wallet Name</label>
            <input
              type="text"
              value={walletName}
              onChange={(e) => setWalletName(e.target.value)}
              placeholder="My Wallet"
              className="w-full bg-vault-card border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm"
            />
          </div>

          <div className="space-y-1">
            <label className="text-sm text-gray-400">Network</label>
            <select
              value={chainId}
              onChange={(e) => setChainId(Number(e.target.value))}
              className="w-full bg-vault-card border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm"
            >
              {CHAINS.map((c) => (
                <option key={c.id} value={c.id}>{c.name}</option>
              ))}
            </select>
          </div>

          <div className="space-y-1">
            <label className="text-sm text-gray-400">Password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Min 8 characters"
              className="w-full bg-vault-card border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm"
            />
          </div>

          <div className="space-y-1">
            <label className="text-sm text-gray-400">Confirm Password</label>
            <input
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              placeholder="Repeat password"
              className="w-full bg-vault-card border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm"
            />
          </div>

          {error && (
            <div className="bg-red-900/30 border border-red-700 rounded-lg px-3 py-2 text-red-400 text-sm">
              {error}
            </div>
          )}

          <button
            onClick={handleCreate}
            disabled={loading}
            className="w-full py-3 bg-vault-gradient text-white rounded-lg font-medium text-sm disabled:opacity-40"
          >
            {loading ? "Creating..." : "Create Wallet"}
          </button>
        </>
      )}

      {step === "created" && wallet && (
        <>
          <div className="bg-green-900/30 border border-green-600 rounded-lg px-4 py-3 text-green-300 text-sm">
            ✅ Wallet created successfully!
          </div>

          <div className="bg-vault-card border border-vault-border rounded-xl p-4">
            <div className="flex justify-between items-center mb-2">
              <span className="text-sm text-gray-400">Name</span>
            </div>
            <p className="text-vault-text font-medium">{wallet.name}</p>
          </div>

          <div className="bg-vault-card border border-vault-border rounded-xl p-4">
            <span className="text-sm text-gray-400">Address</span>
            <p className="text-vault-text font-mono text-sm mt-1 break-all">{wallet.address}</p>
          </div>

          <div className="bg-amber-900/30 border border-amber-600 rounded-lg px-4 py-3 text-amber-300 text-sm">
            ⚠️ IMPORTANT: Back up your wallet! Go to Wallet Details to export your private key. Never share it with anyone.
          </div>

          <div className="flex gap-3">
            <button
              onClick={() => navigate("/dashboard")}
              className="flex-1 py-3 bg-vault-gradient text-white rounded-lg font-medium text-sm"
            >
              Go to Dashboard
            </button>
            <button
              onClick={() => navigate(`/wallet/${wallet.id}`)}
              className="flex-1 py-3 bg-vault-card border border-vault-border text-vault-text rounded-lg font-medium text-sm"
            >
              View Wallet
            </button>
          </div>
        </>
      )}
    </div>
  );
}
