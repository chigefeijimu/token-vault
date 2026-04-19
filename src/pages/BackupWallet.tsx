import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useNavigate, useParams } from "react-router-dom";

interface DecryptResponse {
  mnemonic: string;
}

export function BackupWallet() {
  const navigate = useNavigate();
  const { walletId } = useParams<{ walletId: string }>();
  const [step, setStep] = useState<"password" | "mnemonic">("password");
  const [password, setPassword] = useState("");
  const [mnemonic, setMnemonic] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [confirmed, setConfirmed] = useState(false);

  async function handleDecrypt() {
    if (!password.trim()) { setError("Password is required"); return; }
    if (!walletId) { setError("Wallet ID is missing"); return; }
    setError("");
    setLoading(true);
    try {
      const result = await invoke<DecryptResponse>("decrypt_wallet", {
        id: walletId,
        password,
      });
      const words = result.mnemonic.trim().split(/\s+/);
      if (words.length !== 12) {
        throw new Error(`Expected 12 words, got ${words.length}`);
      }
      setMnemonic(words);
      setStep("mnemonic");
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  function handleContinue() {
    navigate("/dashboard");
  }

  return (
    <div className="p-4 sm:p-6 max-w-xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-vault-text">Backup Wallet</h1>
        <p className="text-sm text-gray-500 mt-0.5">Save your recovery phrase</p>
      </div>

      {step === "password" && (
        <>
          <div className="bg-amber-900/30 border border-amber-600 rounded-lg px-4 py-3 text-amber-300 text-sm">
            ⚠️ Enter your wallet password to reveal the recovery phrase.
          </div>

          <div className="space-y-1">
            <label className="text-sm text-gray-400">Wallet Password</label>
            <input
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              placeholder="Enter password"
              className="w-full bg-vault-card border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm"
              onKeyDown={(e) => e.key === "Enter" && handleDecrypt()}
            />
          </div>

          {error && (
            <div className="bg-red-900/30 border border-red-700 rounded-lg px-3 py-2 text-red-400 text-sm">
              {error}
            </div>
          )}

          <button
            onClick={handleDecrypt}
            disabled={loading}
            className="w-full py-3 bg-vault-gradient text-white rounded-lg font-medium text-sm disabled:opacity-40"
          >
            {loading ? "Decrypting..." : "Show Recovery Phrase"}
          </button>
        </>
      )}

      {step === "mnemonic" && (
        <>
          <div className="bg-amber-900/30 border border-amber-600 rounded-lg px-4 py-3 text-amber-300 text-sm">
            ⚠️ Write down these words and store them safely. Anyone with these words can access your funds.
          </div>

          {/* 3x4 grid for 12 words */}
          <div className="grid grid-cols-3 gap-2">
            {mnemonic.map((word, index) => (
              <div
                key={index}
                className="bg-vault-card border border-vault-border rounded-lg px-3 py-2.5 flex items-center gap-2"
              >
                <span className="text-xs text-gray-500 w-4">{index + 1}.</span>
                <span className="text-vault-text font-medium text-sm">{word}</span>
              </div>
            ))}
          </div>

          {/* Confirmation checkbox */}
          <div className="flex items-start gap-3">
            <input
              type="checkbox"
              id="confirm-backup"
              checked={confirmed}
              onChange={(e) => setConfirmed(e.target.checked)}
              className="mt-1 w-4 h-4 rounded border-vault-border bg-vault-card accent-purple-500"
            />
            <label htmlFor="confirm-backup" className="text-sm text-gray-400 cursor-pointer">
              I've written down my recovery phrase and stored it safely
            </label>
          </div>

          <button
            onClick={handleContinue}
            disabled={!confirmed}
            className="w-full py-3 bg-vault-gradient text-white rounded-lg font-medium text-sm disabled:opacity-40 disabled:cursor-not-allowed"
          >
            Continue to Dashboard
          </button>
        </>
      )}
    </div>
  );
}
