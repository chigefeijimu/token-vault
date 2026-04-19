import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useWalletStore } from "../stores/walletStore";
import { usePendingTxStore } from "../stores/pendingTxStore";
import { CHAINS } from "../types/wallet";

export function SendTransfer() {
  const { wallets } = useWalletStore();
  const addPendingTx = usePendingTxStore((s) => s.addPendingTx);
  const [selectedWalletId, setSelectedWalletId] = useState("");
  const [recipient, setRecipient] = useState("");
  const [amount, setAmount] = useState("");
  const [chainId, setChainId] = useState(1);
  const [tokenAddress, setTokenAddress] = useState("");
  const [customToken, setCustomToken] = useState(false);
  const [gasLimit, setGasLimit] = useState("");
  const [step, setStep] = useState<"form" | "confirm" | "success" | "error">("form");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [txHash, setTxHash] = useState("");

  const selectedWallet = wallets.find((w) => w.id === selectedWalletId);
  const selectedChain = CHAINS.find((c) => c.id === chainId);

  useEffect(() => {
    if (selectedWallet && recipient && amount && !customToken) {
      estimateGas();
    }
  }, [recipient, amount, chainId, selectedWallet]);

  async function estimateGas() {
    if (!selectedWallet || !recipient || !amount) return;
    try {
      const result = await invoke<{ gas_limit: string }>("estimate_gas", {
        from: selectedWallet.address,
        to: recipient,
        value: BigInt(Math.floor(parseFloat(amount) * 1e18)).toString(),
        data: null,
        chainId,
      });
      setGasLimit(result.gas_limit);
    } catch (e) {
      setGasLimit("21000");
    }
  }

  async function getMaxBalance() {
    if (!selectedWallet) return;
    try {
      const result = await invoke<{ balance: string }>("get_balance", {
        address: selectedWallet.address,
        chainId,
      });
      const bal = BigInt(result.balance);
      const gas = BigInt(gasLimit || "21000") * BigInt(30000000000n); // 30 gwei
      const max = bal > gas ? bal - gas : 0n;
      setAmount((Number(max) / 1e18).toFixed(6));
    } catch (e) {
      console.error(e);
    }
  }

  function validateAddress(addr: string): boolean {
    return /^0x[0-9a-fA-F]{40}$/.test(addr);
  }

  async function handleSend() {
    if (!selectedWallet) { setError("Select a wallet"); return; }
    if (!validateAddress(recipient)) { setError("Invalid recipient address"); return; }
    const amountNum = parseFloat(amount);
    if (isNaN(amountNum) || amountNum <= 0) { setError("Invalid amount"); return; }
    setError("");
    setStep("confirm");
  }

  async function confirmSend() {
    if (!selectedWallet) return;
    setLoading(true);
    setError("");
    try {
      const valueWei = BigInt(Math.floor(parseFloat(amount) * 1e18)).toString();
      let result: { tx_hash: string };
      if (customToken && tokenAddress) {
        result = await invoke<{ tx_hash: string }>("send_erc20_token", {
          from: selectedWallet.address,
          to: recipient,
          tokenAddress,
          amount: valueWei,
          chainId,
        });
      } else {
        result = await invoke<{ tx_hash: string }>("send_transaction", {
          from: selectedWallet.address,
          to: recipient,
          value: valueWei,
          chainId,
        });
      }
      setTxHash(result.tx_hash);
      addPendingTx(
        result.tx_hash,
        chainId,
        selectedWallet.address,
        recipient,
        amount
      );
      setStep("success");
    } catch (e: unknown) {
      setError(String(e));
      setStep("error");
    } finally {
      setLoading(false);
    }
  }

  const explorerUrl = selectedChain?.explorerUrl || "";

  return (
    <div className="p-4 sm:p-6 max-w-xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-vault-text">Send</h1>
        <p className="text-sm text-gray-500 mt-0.5">Transfer ETH or tokens</p>
      </div>

      {step === "form" && (
        <>
          {/* Wallet selector */}
          <div className="space-y-1">
            <label className="text-sm text-gray-400">From Wallet</label>
            <select
              value={selectedWalletId}
              onChange={(e) => setSelectedWalletId(e.target.value)}
              className="w-full bg-vault-card border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm"
            >
              <option value="">Select wallet</option>
              {wallets.map((w) => (
                <option key={w.id} value={w.id}>{w.name} ({w.address.slice(0,6)}...{w.address.slice(-4)})</option>
              ))}
            </select>
          </div>

          {/* Chain selector */}
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

          {/* Recipient */}
          <div className="space-y-1">
            <label className="text-sm text-gray-400">Recipient Address</label>
            <input
              type="text"
              value={recipient}
              onChange={(e) => setRecipient(e.target.value)}
              placeholder="0x..."
              className="w-full bg-vault-card border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm font-mono"
            />
          </div>

          {/* Amount */}
          <div className="space-y-1">
            <div className="flex justify-between">
              <label className="text-sm text-gray-400">Amount</label>
              <button
                onClick={getMaxBalance}
                className="text-xs text-purple-400 hover:text-purple-300"
              >
                MAX
              </button>
            </div>
            <div className="relative">
              <input
                type="number"
                step="0.000001"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0.0"
                className="w-full bg-vault-card border border-vault-border rounded-lg px-3 py-2.5 pr-16 text-vault-text text-sm"
              />
              <span className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-500 text-sm">
                {selectedChain?.symbol || "ETH"}
              </span>
            </div>
          </div>

          {/* Token toggle */}
          <div className="flex items-center gap-3">
            <button
              onClick={() => setCustomToken(false)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium ${!customToken ? "bg-vault-gradient text-white" : "bg-vault-card border border-vault-border text-gray-400"}`}
            >
              Native
            </button>
            <button
              onClick={() => setCustomToken(true)}
              className={`px-3 py-1.5 rounded-lg text-xs font-medium ${customToken ? "bg-vault-gradient text-white" : "bg-vault-card border border-vault-border text-gray-400"}`}
            >
              ERC-20
            </button>
          </div>

          {customToken && (
            <div className="space-y-1">
              <label className="text-sm text-gray-400">Token Contract Address</label>
              <input
                type="text"
                value={tokenAddress}
                onChange={(e) => setTokenAddress(e.target.value)}
                placeholder="0x..."
                className="w-full bg-vault-card border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm font-mono"
              />
            </div>
          )}

          {/* Gas limit */}
          <div className="space-y-1">
            <label className="text-sm text-gray-400">Gas Limit</label>
            <input
              type="text"
              value={gasLimit}
              onChange={(e) => setGasLimit(e.target.value)}
              className="w-full bg-vault-card border border-vault-border rounded-lg px-3 py-2.5 text-vault-text text-sm font-mono"
            />
          </div>

          {error && (
            <div className="bg-red-900/30 border border-red-700 rounded-lg px-3 py-2 text-red-400 text-sm">
              {error}
            </div>
          )}

          <button
            onClick={handleSend}
            disabled={!selectedWallet || !recipient || !amount}
            className="w-full py-3 bg-vault-gradient text-white rounded-lg font-medium text-sm disabled:opacity-40"
          >
            Review Transaction
          </button>
        </>
      )}

      {step === "confirm" && selectedWallet && (
        <div className="space-y-4">
          <h2 className="text-lg font-semibold text-vault-text">Confirm Transaction</h2>
          <div className="bg-vault-card border border-vault-border rounded-xl p-4 space-y-3">
            <div className="flex justify-between text-sm">
              <span className="text-gray-400">From</span>
              <span className="text-vault-text font-mono">{selectedWallet.address.slice(0,10)}...{selectedWallet.address.slice(-6)}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-400">To</span>
              <span className="text-vault-text font-mono">{recipient.slice(0,10)}...{recipient.slice(-6)}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-400">Amount</span>
              <span className="text-vault-text font-bold">{amount} {selectedChain?.symbol}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-400">Network</span>
              <span className="text-vault-text">{selectedChain?.name}</span>
            </div>
            <div className="flex justify-between text-sm">
              <span className="text-gray-400">Gas Limit</span>
              <span className="text-vault-text font-mono">{gasLimit}</span>
            </div>
            {customToken && (
              <div className="flex justify-between text-sm">
                <span className="text-gray-400">Token</span>
                <span className="text-vault-text font-mono">{tokenAddress.slice(0,10)}...</span>
              </div>
            )}
          </div>

          {error && (
            <div className="bg-red-900/30 border border-red-700 rounded-lg px-3 py-2 text-red-400 text-sm">
              {error}
            </div>
          )}

          <div className="flex gap-3">
            <button
              onClick={() => setStep("form")}
              className="flex-1 py-3 bg-vault-card border border-vault-border text-vault-text rounded-lg font-medium text-sm"
            >
              Cancel
            </button>
            <button
              onClick={confirmSend}
              disabled={loading}
              className="flex-1 py-3 bg-vault-gradient text-white rounded-lg font-medium text-sm disabled:opacity-40"
            >
              {loading ? "Signing..." : "Confirm & Send"}
            </button>
          </div>
        </div>
      )}

      {step === "success" && (
        <div className="space-y-4 text-center py-8">
          <div className="text-5xl">✅</div>
          <h2 className="text-xl font-bold text-vault-text">Transaction Sent!</h2>
          <p className="text-sm text-gray-400">Your transaction has been broadcast to the network.</p>
          <a
            href={`${explorerUrl}/tx/${txHash}`}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-block text-purple-400 hover:text-purple-300 text-sm font-mono break-all"
          >
            {txHash.slice(0,20)}...
          </a>
          <div className="flex gap-3">
            <button
              onClick={() => { setStep("form"); setAmount(""); setRecipient(""); setTxHash(""); }}
              className="flex-1 py-3 bg-vault-gradient text-white rounded-lg font-medium text-sm"
            >
              Send Another
            </button>
            <a
              href={`${explorerUrl}/tx/${txHash}`}
              target="_blank"
              rel="noopener noreferrer"
              className="flex-1 py-3 bg-vault-card border border-vault-border text-vault-text rounded-lg font-medium text-sm text-center"
            >
              View on Explorer
            </a>
          </div>
        </div>
      )}

      {step === "error" && (
        <div className="space-y-4 text-center py-8">
          <div className="text-5xl">❌</div>
          <h2 className="text-xl font-bold text-vault-text">Transaction Failed</h2>
          <p className="text-sm text-red-400 px-4">{error}</p>
          <div className="flex gap-3">
            <button
              onClick={() => setStep("form")}
              className="flex-1 py-3 bg-vault-gradient text-white rounded-lg font-medium text-sm"
            >
              Try Again
            </button>
            <button
              onClick={() => { setStep("form"); setError(""); }}
              className="flex-1 py-3 bg-vault-card border border-vault-border text-vault-text rounded-lg font-medium text-sm"
            >
              Cancel
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
