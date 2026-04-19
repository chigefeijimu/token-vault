import { useState, useEffect } from 'react';
import { useWalletStore } from '../stores/walletStore';
import type { Chain } from '../types/wallet';

interface SendFormData {
  recipientAddress: string;
  amount: string;
  gasLimit: string;
}

interface FormErrors {
  recipientAddress?: string;
  amount?: string;
  gasLimit?: string;
}

export default function Send() {
  const { activeWallet, activeChain, balances, chains, fetchBalance } = useWalletStore();
  
  const [formData, setFormData] = useState<SendFormData>({
    recipientAddress: '',
    amount: '',
    gasLimit: '21000',
  });
  
  const [errors, setErrors] = useState<FormErrors>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [selectedChain, setSelectedChain] = useState<Chain>(activeChain);
  const [estimatedFee, setEstimatedFee] = useState<string>('0');
  const [maxAmount, setMaxAmount] = useState<string>('0');

  // Fetch balance when wallet or chain changes
  useEffect(() => {
    if (activeWallet) {
      fetchBalance(activeWallet.address, selectedChain.id);
    }
  }, [activeWallet, selectedChain, fetchBalance]);

  // Calculate max amount based on balance
  useEffect(() => {
    if (activeWallet) {
      const balanceKey = `${activeWallet.address}-${selectedChain.id}`;
      const balanceInfo = balances.get(balanceKey);
      if (balanceInfo) {
        setMaxAmount(balanceInfo.balance_formatted);
      }
    }
  }, [activeWallet, selectedChain, balances]);

  // Estimate fee based on gas limit
  useEffect(() => {
    // Simple fee estimation: gasLimit * 0.00000003 (approximate for ETH)
    const gasLimitNum = parseInt(formData.gasLimit) || 0;
    const fee = (gasLimitNum * 0.00000003).toFixed(8);
    setEstimatedFee(fee);
  }, [formData.gasLimit]);

  const validateAddress = (address: string): boolean => {
    // Basic Ethereum address validation
    return /^0x[a-fA-F0-9]{40}$/.test(address);
  };

  const validateForm = (): boolean => {
    const newErrors: FormErrors = {};

    // Validate recipient address
    if (!formData.recipientAddress) {
      newErrors.recipientAddress = 'Recipient address is required';
    } else if (!validateAddress(formData.recipientAddress)) {
      newErrors.recipientAddress = 'Invalid Ethereum address format';
    }

    // Validate amount
    if (!formData.amount) {
      newErrors.amount = 'Amount is required';
    } else {
      const amountNum = parseFloat(formData.amount);
      if (isNaN(amountNum) || amountNum <= 0) {
        newErrors.amount = 'Amount must be a positive number';
      }
      const maxAmountNum = parseFloat(maxAmount);
      if (amountNum > maxAmountNum) {
        newErrors.amount = `Insufficient balance. Max: ${maxAmount}`;
      }
    }

    // Validate gas limit
    if (!formData.gasLimit) {
      newErrors.gasLimit = 'Gas limit is required';
    } else {
      const gasLimitNum = parseInt(formData.gasLimit);
      if (isNaN(gasLimitNum) || gasLimitNum < 21000) {
        newErrors.gasLimit = 'Gas limit must be at least 21000';
      }
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleInputChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => {
    const { name, value } = e.target;
    setFormData((prev) => ({
      ...prev,
      [name]: value,
    }));
    // Clear error when user starts typing
    if (errors[name as keyof FormErrors]) {
      setErrors((prev) => ({
        ...prev,
        [name]: undefined,
      }));
    }
  };

  const handleMaxClick = () => {
    const fee = parseFloat(estimatedFee) || 0;
    const max = parseFloat(maxAmount) - fee;
    if (max > 0) {
      setFormData((prev) => ({
        ...prev,
        amount: max.toFixed(8),
      }));
    } else {
      setFormData((prev) => ({
        ...prev,
        amount: '0',
      }));
    }
    setErrors((prev) => ({
      ...prev,
      amount: undefined,
    }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!validateForm() || !activeWallet) {
      return;
    }

    setIsSubmitting(true);

    try {
      // In a real implementation, this would call the Tauri command to send the transaction
      // For now, we'll simulate the transaction sending
      console.log('Sending transaction:', {
        from: activeWallet.address,
        to: formData.recipientAddress,
        amount: formData.amount,
        gasLimit: formData.gasLimit,
        chainId: selectedChain.id,
      });

      // Simulate transaction submission
      await new Promise((resolve) => setTimeout(resolve, 1000));

      // Reset form after successful submission
      setFormData({
        recipientAddress: '',
        amount: '',
        gasLimit: '21000',
      });

      alert('Transaction submitted successfully!');
    } catch (error) {
      console.error('Transaction failed:', error);
      alert('Transaction failed. Please try again.');
    } finally {
      setIsSubmitting(false);
    }
  };

  const balanceKey = activeWallet
    ? `${activeWallet.address}-${selectedChain.id}`
    : '';
  const currentBalance = balances.get(balanceKey);

  return (
    <div className="min-h-screen bg-gray-900 text-white p-6">
      <div className="max-w-2xl mx-auto">
        {/* Header */}
        <div className="mb-8">
          <h1 className="text-3xl font-bold mb-2">Send Transaction</h1>
          <p className="text-gray-400">
            Transfer {selectedChain.nativeCurrency?.symbol || selectedChain.symbol} to another address
          </p>
        </div>

        {/* Wallet Status */}
        {!activeWallet ? (
          <div className="bg-yellow-900/30 border border-yellow-600 rounded-lg p-6 text-center">
            <p className="text-yellow-400">Please select or create a wallet first</p>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-6">
            {/* Chain Selector */}
            <div className="bg-gray-800 rounded-lg p-4">
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Network
              </label>
              <select
                value={selectedChain.id}
                onChange={(e) => {
                  const chain = chains.find((c) => c.id === parseInt(e.target.value));
                  if (chain) {
                    setSelectedChain(chain);
                  }
                }}
                className="w-full bg-gray-700 border border-gray-600 rounded-lg px-4 py-3 text-white focus:outline-none focus:ring-2 focus:ring-blue-500"
              >
                {chains.map((chain) => (
                  <option key={chain.id} value={chain.id}>
                    {chain.name} ({chain.symbol})
                  </option>
                ))}
              </select>
            </div>

            {/* From Address (Read-only) */}
            <div className="bg-gray-800 rounded-lg p-4">
              <label className="block text-sm font-medium text-gray-300 mb-2">
                From
              </label>
              <div className="flex items-center space-x-3">
                <div className="flex-1 bg-gray-700 rounded-lg px-4 py-3 text-gray-400 font-mono text-sm break-all">
                  {activeWallet.address}
                </div>
                <div className="bg-gray-700 rounded-lg px-4 py-3 text-white">
                  <span className="text-sm text-gray-400">Balance: </span>
                  <span className="font-semibold">
                    {currentBalance?.balance_formatted || '0'} {selectedChain.symbol}
                  </span>
                </div>
              </div>
            </div>

            {/* Recipient Address */}
            <div className="bg-gray-800 rounded-lg p-4">
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Recipient Address
              </label>
              <input
                type="text"
                name="recipientAddress"
                value={formData.recipientAddress}
                onChange={handleInputChange}
                placeholder="0x..."
                className={`w-full bg-gray-700 border ${
                  errors.recipientAddress ? 'border-red-500' : 'border-gray-600'
                } rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 font-mono`}
              />
              {errors.recipientAddress && (
                <p className="mt-2 text-sm text-red-400">{errors.recipientAddress}</p>
              )}
            </div>

            {/* Amount */}
            <div className="bg-gray-800 rounded-lg p-4">
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Amount
              </label>
              <div className="flex items-center space-x-3">
                <input
                  type="text"
                  name="amount"
                  value={formData.amount}
                  onChange={handleInputChange}
                  placeholder="0.0"
                  className={`flex-1 bg-gray-700 border ${
                    errors.amount ? 'border-red-500' : 'border-gray-600'
                  } rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500`}
                />
                <span className="text-gray-400 font-semibold">{selectedChain.symbol}</span>
                <button
                  type="button"
                  onClick={handleMaxClick}
                  className="bg-blue-600 hover:bg-blue-700 text-white px-4 py-3 rounded-lg font-medium transition-colors"
                >
                  MAX
                </button>
              </div>
              {errors.amount && (
                <p className="mt-2 text-sm text-red-400">{errors.amount}</p>
              )}
            </div>

            {/* Gas Limit */}
            <div className="bg-gray-800 rounded-lg p-4">
              <label className="block text-sm font-medium text-gray-300 mb-2">
                Gas Limit
              </label>
              <input
                type="text"
                name="gasLimit"
                value={formData.gasLimit}
                onChange={handleInputChange}
                placeholder="21000"
                className={`w-full bg-gray-700 border ${
                  errors.gasLimit ? 'border-red-500' : 'border-gray-600'
                } rounded-lg px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500`}
              />
              {errors.gasLimit && (
                <p className="mt-2 text-sm text-red-400">{errors.gasLimit}</p>
              )}
              <p className="mt-2 text-xs text-gray-500">
                Standard ETH transfer: 21000, Token transfers: ~65000-100000
              </p>
            </div>

            {/* Fee Estimation */}
            <div className="bg-gray-800 rounded-lg p-4">
              <div className="flex justify-between items-center">
                <span className="text-gray-400">Estimated Network Fee</span>
                <span className="font-semibold">
                  ~{estimatedFee} {selectedChain.symbol}
                </span>
              </div>
            </div>

            {/* Total */}
            <div className="bg-gray-800 rounded-lg p-4 border border-blue-500/30">
              <div className="flex justify-between items-center">
                <span className="text-lg font-medium">Total</span>
                <span className="text-xl font-bold">
                  {(parseFloat(formData.amount || '0') + parseFloat(estimatedFee || '0')).toFixed(8)}{' '}
                  {selectedChain.symbol}
                </span>
              </div>
            </div>

            {/* Submit Button */}
            <button
              type="submit"
              disabled={isSubmitting}
              className={`w-full py-4 rounded-lg font-semibold text-lg transition-all ${
                isSubmitting
                  ? 'bg-gray-600 cursor-not-allowed'
                  : 'bg-blue-600 hover:bg-blue-700'
              }`}
            >
              {isSubmitting ? 'Processing...' : 'Send Transaction'}
            </button>

            {/* Warning */}
            <p className="text-center text-sm text-gray-500">
              Please double-check the recipient address. Transactions cannot be reversed.
            </p>
          </form>
        )}
      </div>
    </div>
  );
}
