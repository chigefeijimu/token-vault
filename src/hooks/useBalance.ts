import { invoke } from '@tauri-apps/api/core'
import { useState, useCallback } from 'react'
import type { WalletBalances } from '../types/balance'
import { CHAINS, type Chain } from '../types/wallet'

interface BalanceResult {
  success: boolean
  data?: WalletBalances
  error?: string
}

export function useBalance() {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const getNativeBalance = useCallback(async (address: string, chainId: number): Promise<BalanceResult> => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<BalanceResult>('get_native_balance', { address, chainId })
      return result
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      setError(errorMsg)
      return { success: false, error: errorMsg }
    } finally {
      setLoading(false)
    }
  }, [])

  const getErc20Balances = useCallback(async (address: string, chainId: number, tokenAddresses: string[]): Promise<BalanceResult> => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<BalanceResult>('get_erc20_balances', { address, chainId, tokenAddresses })
      return result
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      setError(errorMsg)
      return { success: false, error: errorMsg }
    } finally {
      setLoading(false)
    }
  }, [])

  const getAllBalances = useCallback(async (address: string, chainId: number, tokenAddresses?: string[]): Promise<BalanceResult> => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<BalanceResult>('get_wallet_balances', { 
        address, 
        chainId, 
        tokenAddresses: tokenAddresses || [] 
      })
      return result
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err)
      setError(errorMsg)
      return { success: false, error: errorMsg }
    } finally {
      setLoading(false)
    }
  }, [])

  const getSupportedChains = useCallback((): Chain[] => {
    return CHAINS
  }, [])

  return {
    loading,
    error,
    getNativeBalance,
    getErc20Balances,
    getAllBalances,
    getSupportedChains,
  }
}