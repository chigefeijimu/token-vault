import { useState, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"
import { useTransactionStore } from "../stores/transactionStore"
import type {
  TransactionRecord,
  TransactionHistoryFilter,
  TransactionHistoryResult,
  TransactionReceiptRust,
} from "../types/transaction"
import { parseRustReceipt } from "../types/transaction"

const PAGE_SIZE = 20
const CACHE_TTL_MS = 60_000 // 1 minute

interface UseTransactionHistoryResult {
  transactions: TransactionRecord[]
  isLoading: boolean
  error: string | null
  hasMore: boolean
  totalCount: number
  fetchHistory: (filter: TransactionHistoryFilter) => Promise<void>
  loadMore: (filter: TransactionHistoryFilter) => Promise<void>
  refresh: (filter: TransactionHistoryFilter) => Promise<void>
  clearCache: (address: string, chainId: number) => void
}

// Simulates fetching transactions for an address by polling for recent blocks
// The Rust backend get_transaction_receipt is called per tx hash when needed.
// For history, we use eth_getLogs via the RPC endpoint.
async function fetchTxsFromRpc(
  address: string,
  chainId: number,
  page: number,
  pageSize: number
): Promise<TransactionHistoryResult> {
  try {
    const result = await invoke<TransactionHistoryResult>("get_transaction_history", {
      address,
      chainId,
      page,
      pageSize,
    })
    return result
  } catch {
    // Fallback: try the direct receipt approach for a single hash if needed
    return {
      transactions: [],
      totalCount: 0,
      page,
      pageSize,
      hasMore: false,
    }
  }
}

// Converts raw RPC log entries to TransactionRecord
function rpcLogsToRecords(
  logs: Array<{
    transaction_hash: string
    block_number: number
    block_hash: string
    timestamp: number
    from: string
    to: string
    value: string
    gas_used: string
    gas_price: string
    status: string
  }>,
  chainId: number
): TransactionRecord[] {
  return logs.map(log => parseRustReceipt(log as TransactionReceiptRust, chainId))
}

export function useTransactionHistory(): UseTransactionHistoryResult {
  const store = useTransactionStore()
  const [totalCount, setTotalCount] = useState(0)
  const [hasMore, setHasMore] = useState(false)

  const fetchHistory = useCallback(async (filter: TransactionHistoryFilter) => {
    const { address, chainId, page = 1, pageSize = PAGE_SIZE } = filter
    const cacheKey = store.getCacheKey(address, chainId)
    const cached = store.historyCache[cacheKey]

    // Return from cache if fresh
    if (cached && Date.now() - cached.fetchedAt < CACHE_TTL_MS) {
      setTotalCount(cached.transactions.length)
      setHasMore(false)
      return
    }

    store.setFetching(true)
    store.setError(null)

    try {
      const result = await fetchTxsFromRpc(address, chainId, page, pageSize)
      const records = rpcLogsToRecords(result.transactions as any, chainId)
      store.setHistory(address, chainId, records)
      setTotalCount(result.totalCount)
      setHasMore(result.hasMore)
    } catch (err) {
      store.setError(err instanceof Error ? err.message : String(err))
    } finally {
      store.setFetching(false)
    }
  }, [store])

  const loadMore = useCallback(async (filter: TransactionHistoryFilter) => {
    const { address, chainId, page = 1, pageSize = PAGE_SIZE } = filter
    store.setFetching(true)
    try {
      const nextPage = page + 1
      const result = await fetchTxsFromRpc(address, chainId, nextPage, pageSize)
      const records = rpcLogsToRecords(result.transactions as any, chainId)
      store.appendTransactions(address, chainId, records)
      setHasMore(result.hasMore)
    } catch (err) {
      store.setError(err instanceof Error ? err.message : String(err))
    } finally {
      store.setFetching(false)
    }
  }, [store])

  const refresh = useCallback(async (filter: TransactionHistoryFilter) => {
    const { address, chainId } = filter
    store.clearHistory(address, chainId)
    await fetchHistory(filter)
  }, [fetchHistory, store])

  const clearCache = useCallback((address: string, chainId: number) => {
    store.clearHistory(address, chainId)
    setTotalCount(0)
    setHasMore(false)
  }, [store])

  return {
    transactions: [], // resolved from store.getHistory in component
    isLoading: store.isFetching,
    error: store.error,
    hasMore,
    totalCount,
    fetchHistory,
    loadMore,
    refresh,
    clearCache,
  }
}

// Simpler hook that reads directly from store
export function useTransactionStoreData(address: string, chainId: number) {
  const store = useTransactionStore()
  const cached = store.getHistory(address, chainId)
  return {
    transactions: cached ?? [],
    pendingTransactions: store.pendingTransactions,
    isFetching: store.isFetching,
    error: store.error,
  }
}

