import { useState, FormEvent } from 'react';
import { useWalletStore } from '../stores/walletStore';
import type { Chain } from '../types/wallet';

export function SendPage() {
  const { chains, activeChain, activeWallet, setActiveChain, balances } = useWalletStore();
  
  const [recipientAddress, setRecipientAddress] = useState('');
  const [amount, setAmount] = useState('');
  const [selectedChain, setSelectedChain] = useState<Chain>(activeChain);
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<string | null>(null);

  // Get current balance for selected chain
  const currentBalanceKey = activeWallet ? `${activeWallet.address}-${selectedChain.id}` : null;
  const currentBalance = currentBalanceKey ? balances.get(currentBalanceKey) : null;

  // Validate Ethereum address (basic check)
  const isValidAddress = (address: string): boolean => {
    return /^0x[a-fA-F0-9]{40}$/.test(address);
  };

  // Validate amount
  const isValidAmount = (value: string): boolean => {
    if (!value || parseFloat(value) <= 0) return false;
    if (currentBalance) {
      return parseFloat(value) <= parseFloat(currentBalance.balance_formatted);
    }
    return true;
  };

  const handleChainChange = (chainId: number) => {
    const chain = chains.find(c => c.id === chainId);
    if (chain) {
      setSelectedChain(chain);
      setActiveChain(chain);
    }
  };

  const handleMaxClick = () => {
    if (currentBalance) {
      // Leave some for gas - rough estimate
      const maxAmount = Math.max(0, parseFloat(currentBalance.balance_formatted) - 0.001);
      setAmount(maxAmount.toFixed(6));
    }
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();
    setError(null);
    setSuccess(null);

    // Validation
    if (!recipientAddress) {
      setError('Recipient address is required');
      return;
    }
    if (!isValidAddress(recipientAddress)) {
      setError('Invalid recipient address format');
      return;
    }
    if (!amount) {
      setError('Amount is required');
      return;
    }
    if (!isValidAmount(amount)) {
      setError('Invalid amount or insufficient balance');
      return;
    }
    if (!activeWallet) {
      setError('No wallet selected');
      return;
    }

    setIsSubmitting(true);

    try {
      // TODO: Implement actual transaction submission via Tauri command
      // This would call a send_transaction command that signs and broadcasts
      console.log('Sending transaction:', {
        from: activeWallet.address,
        to: recipientAddress,
        amount,
        chainId: selectedChain.id,
        symbol: selectedChain.symbol,
      });

      // Simulate transaction submission
      await new Promise(resolve => setTimeout(resolve, 1500));

      setSuccess(`Transaction submitted successfully! ${amount} ${selectedChain.symbol} to ${recipientAddress.slice(0, 10)}...${recipientAddress.slice(-4)}`);
      
      // Reset form
      setRecipientAddress('');
      setAmount('');
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Transaction failed');
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleClear = () => {
    setRecipientAddress('');
    setAmount('');
    setError(null);
    setSuccess(null);
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white p-6">
      <div className="max-w-2xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-white mb-2">Send</h1>
          <p className="text-gray-400">Transfer {selectedChain.symbol} to another address</p>
        </div>

        {/* Sender Info */}
        <div className="bg-gray-800 rounded-lg p-4 mb-6">
          <div className="text-sm text-gray-400 mb-1">From</div>
          <div className="flex items-center justify-between">
            <div>
              <div className="font-medium text-white">
                {activeWallet?.name || 'No wallet selected'}
              </div>
              <div className="text-sm text-gray-400 font-mono">
                {activeWallet?.address
                  ? `${activeWallet.address.slice(0, 10)}...${activeWallet.address.slice(-4)}`
                  : 'No address'}
              </div>
            </div>
            <div className="text-right">
              <div className="text-lg font-semibold text-white">
                {currentBalance ? parseFloat(currentBalance.balance_formatted).toFixed(6) : '0.000000'} {selectedChain.symbol}
              </div>
              <div className="text-sm text-gray-400">
                Balance
              </div>
            </div>
          </div>
        </div>

        {/* Send Form */}
        <form onSubmit={handleSubmit} className="bg-gray-800 rounded-lg p-6 space-y-6">
          {/* Chain Selector */}
          <div>
            <label htmlFor="chain" className="block text-sm font-medium text-gray-300 mb-2">
              Network
            </label>
            <select
              id="chain"
              value={selectedChain.id}
              onChange={(e) => handleChainChange(Number(e.target.value))}
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              {chains.filter(chain => chain.isSupported).map((chain) => (
                <option key={chain.id} value={chain.id}>
                  {chain.name} ({chain.symbol})
                </option>
              ))}
            </select>
          </div>

          {/* Recipient Address */}
          <div>
            <label htmlFor="recipient" className="block text-sm font-medium text-gray-300 mb-2">
              Recipient Address
            </label>
            <input
              type="text"
              id="recipient"
              value={recipientAddress}
              onChange={(e) => setRecipientAddress(e.target.value)}
              placeholder="0x..."
              className="w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent font-mono"
            />
          </div>

          {/* Amount */}
          <div>
            <label htmlFor="amount" className="block text-sm font-medium text-gray-300 mb-2">
              Amount
            </label>
            <div className="relative">
              <input
                type="text"
                id="amount"
                value={amount}
                onChange={(e) => setAmount(e.target.value)}
                placeholder="0.0"
                className="w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white placeholder-gray-400 focus:ring-2 focus:ring-blue-500 focus:border-transparent pr-20"
              />
              <button
                type="button"
                onClick={handleMaxClick}
                className="absolute right-2 top-1/2 -translate-y-1/2 bg-blue-600 hover:bg-blue-700 text-white text-xs font-medium px-3 py-1 rounded"
              >
                MAX
              </button>
            </div>
            <div className="mt-2 text-sm text-gray-400">
              {currentBalance ? (
                <span>
                  Available: {parseFloat(currentBalance.balance_formatted).toFixed(6)} {selectedChain.symbol}
                </span>
              ) : (
                <span>Loading balance...</span>
              )}
            </div>
          </div>

          {/* Error Message */}
          {error && (
            <div className="bg-red-500/20 border border-red-500 rounded-lg p-4">
              <p className="text-red-400 text-sm">{error}</p>
            </div>
          )}

          {/* Success Message */}
          {success && (
            <div className="bg-green-500/20 border border-green-500 rounded-lg p-4">
              <p className="text-green-400 text-sm">{success}</p>
            </div>
          )}

          {/* Action Buttons */}
          <div className="flex gap-4">
            <button
              type="button"
              onClick={handleClear}
              className="flex-1 bg-gray-700 hover:bg-gray-600 text-white font-medium py-3 rounded-lg transition-colors"
            >
              Clear
            </button>
            <button
              type="submit"
              disabled={isSubmitting || !activeWallet}
              className="flex-1 bg-blue-600 hover:bg-blue-700 disabled:bg-gray-600 disabled:cursor-not-allowed text-white font-medium py-3 rounded-lg transition-colors"
            >
              {isSubmitting ? (
                <span className="flex items-center justify-center gap-2">
                  <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
                    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                    <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                  </svg>
                  Sending...
                </span>
              ) : (
                'Send'
              )}
            </button>
          </div>
        </form>

        {/* Transaction Details Preview */}
        {recipientAddress && amount && isValidAddress(recipientAddress) && (
          <div className="mt-6 bg-gray-800/50 rounded-lg p-4 border border-gray-700">
            <h3 className="text-sm font-medium text-gray-300 mb-3">Transaction Preview</h3>
            <div className="space-y-2 text-sm">
              <div className="flex justify-between">
                <span className="text-gray-400">Network</span>
                <span className="text-white">{selectedChain.name}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">To</span>
                <span className="text-white font-mono">
                  {recipientAddress.slice(0, 10)}...{recipientAddress.slice(-4)}
                </span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-400">Amount</span>
                <span className="text-white font-semibold">{amount} {selectedChain.symbol}</span>
              </div>
            </div>
          </div>
        )}

        {/* Warning */}
        <div className="mt-6 bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-4">
          <p className="text-yellow-400 text-sm">
            <strong>Note:</strong> Transactions on blockchain networks are irreversible. Please double-check the recipient address before sending.
          </p>
        </div>
      </div>
    </div>
  );
}

export default SendPage;
