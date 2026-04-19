import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useWalletStore } from '../stores/walletStore';

export interface ExportKeystoreResult {
  success: boolean;
  privateKey?: string;
  error?: string;
}

export function useWalletExport() {
  const [isExporting, setIsExporting] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const { wallets } = useWalletStore();
  const activeWallet = wallets[0]; // TEMP FIX: use first wallet

  const exportKeystore = useCallback(async (
    walletId: string,
    password: string
  ): Promise<ExportKeystoreResult> => {
    if (!walletId) {
      return {
        success: false,
        error: 'No wallet selected',
      };
    }

    setIsExporting(true);
    setError(null);

    try {
      const privateKey = await invoke<string>('export_keystore', {
        walletId,
        password,
      });

      return {
        success: true,
        privateKey,
      };
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      setError(errorMessage);
      return {
        success: false,
        error: errorMessage,
      };
    } finally {
      setIsExporting(false);
    }
  }, []);

  const exportActiveWallet = useCallback(async (
    password: string
  ): Promise<ExportKeystoreResult> => {
    if (!activeWallet) {
      return {
        success: false,
        error: 'No active wallet',
      };
    }
    return exportKeystore(activeWallet.id, password);
  }, [activeWallet, exportKeystore]);

  return {
    exportKeystore,
    exportActiveWallet,
    isExporting,
    error,
    clearError: () => setError(null),
  };
}
