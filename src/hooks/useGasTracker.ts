import { useState, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { GasInfo, GasEstimate, GasOptimization, FeeBreakdown, GasPriority } from '@/types'

interface UseGasTrackerResult {
  gasInfo: GasInfo | null
  isLoading: boolean
  error: string | null
  refresh: () => Promise<void>
  estimateGas: (
    from: string,
    to: string,
    data?: string,
    value?: string
  ) => Promise<GasEstimate | null>
  getOptimization: (
    gasLimit?: string
  ) => Promise<GasOptimization | null>
  getFeeBreakdown: (
    gasLimit: string,
    priority: GasPriority,
    tokenUsdPrice?: number
  ) => Promise<FeeBreakdown | null>
}

export function useGasTracker(
  chainId: number,
  rpcUrl: string,
  autoRefresh: boolean = false,
  refreshInterval: number = 30000
): UseGasTrackerResult {
  const [gasInfo, setGasInfo] = useState<GasInfo | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const refresh = useCallback(async () => {
    setIsLoading(true)
    setError(null)
    
    try {
      const info = await invoke<GasInfo>('get_gas_prices', {
        chainId,
        rpcUrl,
      })
      setGasInfo(info)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsLoading(false)
    }
  }, [chainId, rpcUrl])

  const estimateGas = useCallback(
    async (
      from: string,
      to: string,
      data?: string,
      value?: string
    ): Promise<GasEstimate | null> => {
      try {
        const estimate = await invoke<GasEstimate>('estimate_transaction_gas', {
          chainId,
          rpcUrl,
          from,
          to,
          data,
          value,
          gasLimit: null,
        })
        return estimate
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err))
        return null
      }
    },
    [chainId, rpcUrl]
  )

  const getOptimization = useCallback(
    async (gasLimit?: string): Promise<GasOptimization | null> => {
      try {
        const optimization = await invoke<GasOptimization>('get_gas_optimization', {
          chainId,
          rpcUrl,
          transactionGasLimit: gasLimit,
        })
        return optimization
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err))
        return null
      }
    },
    [chainId, rpcUrl]
  )

  const getFeeBreakdown = useCallback(
    async (
      gasLimit: string,
      priority: GasPriority,
      tokenUsdPrice?: number
    ): Promise<FeeBreakdown | null> => {
      try {
        const breakdown = await invoke<FeeBreakdown>('get_fee_breakdown', {
          chainId,
          rpcUrl,
          gasLimit,
          priority,
          tokenUsdPrice: tokenUsdPrice ?? null,
        })
        return breakdown
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err))
        return null
      }
    },
    [chainId, rpcUrl]
  )

  useEffect(() => {
    if (autoRefresh) {
      refresh()
      const interval = setInterval(refresh, refreshInterval)
      return () => clearInterval(interval)
    }
  }, [autoRefresh, refresh, refreshInterval])

  return {
    gasInfo,
    isLoading,
    error,
    refresh,
    estimateGas,
    getOptimization,
    getFeeBreakdown,
  }
}

// Hook for quick gas price check
export function useQuickGasCheck(chainId: number, rpcUrl: string) {
  const [prices, setPrices] = useState<{
    low: string
    medium: string
    high: string
  } | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  const check = useCallback(async () => {
    setIsLoading(true)
    try {
      const info = await invoke<{ gas_prices: typeof prices }>('get_gas_prices', {
        chainId,
        rpcUrl,
      })
      setPrices(info.gas_prices)
    } catch (err) {
      console.error('Failed to fetch gas prices:', err)
    } finally {
      setIsLoading(false)
    }
  }, [chainId, rpcUrl])

  useEffect(() => {
    check()
  }, [check])

  return { prices, isLoading, check }
}