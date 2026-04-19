/**
 * WalletConnect v2 Integration Placeholder
 * 
 * This module provides minimal WalletConnect v2 integration.
 * 
 * Requirements:
 * - Project ID from https://cloud.walletconnect.com/
 * - Package: @walletconnect/ethereum-provider
 * 
 * To fully implement:
 * 1. Register at cloud.walletconnect.com for a Project ID
 * 2. Replace WALLETCONNECT_PROJECT_ID placeholder
 * 3. Implement event handlers for connection/disconnection
 * 4. Bridge WC events to Rust backend for transaction signing
 */
import { useState, useCallback } from 'react';

// WalletConnect Project ID - Replace with your own from cloud.walletconnect.com
const WALLETCONNECT_PROJECT_ID = 'YOUR_PROJECT_ID_HERE';

// Chain IDs supported by TokenVault
const SUPPORTED_CHAIN_IDS = [1, 56, 137, 42161, 10, 43114];

interface WalletConnectState {
  isConnected: boolean;
  accounts: string[];
  chainId: number | null;
}

export function WalletConnect() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [wcState, setWcState] = useState<WalletConnectState>({
    isConnected: false,
    accounts: [],
    chainId: null,
  });

  // Dynamic import of WalletConnect provider
  const openWalletConnectModal = useCallback(async () => {
    if (WALLETCONNECT_PROJECT_ID === 'YOUR_PROJECT_ID_HERE') {
      setError('WalletConnect Project ID not configured. Please register at cloud.walletconnect.com');
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      const { EthereumProvider } = await import('@walletconnect/ethereum-provider');

      const provider = await EthereumProvider.init({
        projectId: WALLETCONNECT_PROJECT_ID,
        chains: [1], // Ethereum mainnet as default
        showQrModal: true,
        methods: ['eth_sendTransaction', 'personal_sign', 'eth_signTypedData'],
        events: ['accountsChanged', 'chainChanged'],
        metadata: {
          name: 'TokenVault',
          description: 'TokenVault EVM Wallet',
          url: 'https://tokenvault.app',
          icons: ['https://tokenvault.app/icon.png'],
        },
      });

      // Enable session (triggers QR modal)
      await provider.enable();

      // Set up event listeners
      provider.on('accountsChanged', (accounts: string[]) => {
        setWcState(prev => ({ ...prev, accounts, isConnected: accounts.length > 0 }));
      });

      provider.on('chainChanged', (chainId: string) => {
        setWcState(prev => ({ ...prev, chainId: parseInt(chainId, 16) }));
      });

      // Update state with connection info
      const accounts = await provider.request({ method: 'eth_accounts' }) as string[];
      const chainId = await provider.request({ method: 'eth_chainId' }) as string;

      setWcState({
        isConnected: accounts.length > 0,
        accounts,
        chainId: parseInt(chainId, 16),
      });
      // Store provider reference if needed for later use
      (window as any).walletConnectProvider = provider;

    } catch (err) {
      console.error('WalletConnect connection error:', err);
      setError(err instanceof Error ? err.message : 'Failed to connect');
    } finally {
      setIsLoading(false);
    }
  }, []);

  const disconnect = useCallback(async () => {
    const provider = (window as any).walletConnectProvider;
    if (provider) {
      await provider.disconnect();
      setWcState({ isConnected: false, accounts: [], chainId: null });
      (window as any).walletConnectProvider = null;
    }
  }, []);

  return (
    <div className="p-4 sm:p-6 max-w-2xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3">
        <div>
          <h1 className="text-2xl font-bold text-vault-text">WalletConnect</h1>
          <p className="text-sm text-gray-500 mt-0.5">
            Connect to dApps via WalletConnect v2
          </p>
        </div>
      </div>

      {/* Connection Status Card */}
      <div className="bg-vault-card rounded-xl p-6 border border-vault-border">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold text-vault-text">Connection Status</h2>
          <div className={`px-3 py-1 rounded-full text-sm font-medium ${
            wcState.isConnected 
              ? 'bg-green-500/20 text-green-400' 
              : 'bg-gray-500/20 text-gray-400'
          }`}>
            {wcState.isConnected ? 'Connected' : 'Disconnected'}
          </div>
        </div>

        {wcState.isConnected ? (
          <div className="space-y-3">
            <div className="flex justify-between items-center py-2 border-b border-vault-border">
              <span className="text-gray-400 text-sm">Address</span>
              <span className="text-vault-text font-mono text-sm">
                {wcState.accounts[0] 
                  ? `${wcState.accounts[0].slice(0, 6)}...${wcState.accounts[0].slice(-4)}`
                  : 'N/A'}
              </span>
            </div>
            <div className="flex justify-between items-center py-2 border-b border-vault-border">
              <span className="text-gray-400 text-sm">Chain ID</span>
              <span className="text-vault-text font-mono text-sm">
                {wcState.chainId ?? 'N/A'}
              </span>
            </div>
            <div className="flex justify-between items-center py-2">
              <span className="text-gray-400 text-sm">Supported Chains</span>
              <span className="text-vault-text text-sm">
                {SUPPORTED_CHAIN_IDS.length} chains
              </span>
            </div>

            <button
              onClick={disconnect}
              className="w-full mt-4 px-4 py-2 bg-red-500/20 text-red-400 rounded-lg text-sm font-medium hover:bg-red-500/30 transition"
            >
              Disconnect
            </button>
          </div>
        ) : (
          <div className="text-center py-6">
            <p className="text-gray-400 mb-4">
              Connect your wallet to interact with dApps
            </p>
            
            {error && (
              <div className="mb-4 p-3 bg-red-500/20 border border-red-500/30 rounded-lg">
                <p className="text-red-400 text-sm">{error}</p>
              </div>
            )}

            <button
              onClick={openWalletConnectModal}
              disabled={isLoading}
              className="px-6 py-3 bg-vault-gradient text-white rounded-lg font-medium hover:opacity-90 transition disabled:opacity-50"
            >
              {isLoading ? (
                <span className="flex items-center gap-2">
                  <div className="h-4 w-4 border-2 border-white border-t-transparent rounded-full animate-spin" />
                  Connecting...
                </span>
              ) : (
                'Connect Wallet'
              )}
            </button>

            <p className="mt-4 text-xs text-gray-500">
              Requires WalletConnect Project ID from cloud.walletconnect.com
            </p>
          </div>
        )}
      </div>

      {/* Info Card */}
      <div className="bg-vault-card rounded-xl p-4 border border-vault-border">
        <h3 className="text-sm font-semibold text-vault-text mb-2">About WalletConnect v2</h3>
        <ul className="text-xs text-gray-400 space-y-1">
          <li>• WalletConnect is an open protocol for connecting dApps to mobile wallets</li>
          <li>• Scan QR code with your mobile wallet to connect</li>
          <li>• Supports Ethereum, BNB Chain, Polygon, Arbitrum, Optimism, Avalanche</li>
          <li>• Version 2 offers improved session persistence and multi-chain support</li>
        </ul>
      </div>
    </div>
  );
}

export default WalletConnect;
