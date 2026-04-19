import React, { useState, useEffect, useCallback } from "react"
import { invoke } from "@tauri-apps/api/core"
import { CHAINS } from "../types/wallet"
import { useTransactionStoreData } from "../hooks/useTransactionHistory"
import { usePendingTxStore } from "../stores/pendingTxStore"
import type { TransactionRecord } from "../types/transaction"
import { TransactionStatusBadge } from "./TransactionStatusBadge"

const PAGE_SIZE = 20

interface TransactionHistoryProps {
  address?: string
  chainId?: number
}

function shortenAddress(addr: string): string {
  if (!addr || addr.length < 12) return addr
  return `${addr.slice(0, 6)}...${addr.slice(-4)}`
}

function formatTimestamp(ts: number): string {
  const d = new Date(ts)
  const now = Date.now()
  const diff = now - ts
  if (diff < 60_000) return "Just now"
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`
  if (diff < 86_400_000) return `${Math.floor(diff / 3_600_000)}h ago`
  return d.toLocaleDateString()
}

function formatValue(value: string, symbol: string): string {
  const num = parseFloat(value)
  if (isNaN(num)) return `0 ${symbol}`
  if (num >= 1) return `${num.toFixed(4)} ${symbol}`
  return `${num.toFixed(8)} ${symbol}`
}

export const TransactionHistory: React.FC<TransactionHistoryProps> = ({
  address: initialAddress = "",
  chainId: initialChainId = 1,
}) => {
  const [address, setAddress] = useState(initialAddress)
  const [selectedChain, setSelectedChain] = useState(initialChainId)
  const [transactions, setTransactions] = useState<TransactionRecord[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [page, setPage] = useState(1)
  const [hasMore, setHasMore] = useState(false)
  const [submittedAddress, setSubmittedAddress] = useState(initialAddress)

  const { pendingTransactions } = useTransactionStoreData(submittedAddress, selectedChain)
  const pendingTxsByChain = usePendingTxStore((s) => s.getPendingTxsByChain(selectedChain))

  const chain = CHAINS.find(c => c.id === selectedChain) ?? CHAINS[0]
  const symbol = chain.symbol

  const fetchHistory = useCallback(async (addr: string, cid: number, pageNum: number, append: boolean) => {
    if (!addr || addr.length < 20) return
    setIsLoading(true)
    setError(null)
    try {
      const result = await invoke<{
        transactions: TransactionRecord[]
        totalCount: number
        hasMore: boolean
      }>("get_transaction_history", {
        address: addr,
        chainId: cid,
        page: pageNum,
        pageSize: PAGE_SIZE,
      })
      const records = result.transactions.map((tx: any) => ({
        ...tx,
        timestamp: tx.timestamp * 1000,
        chainId: cid,
      }))
      if (append) {
        setTransactions(prev => {
          const seen = new Set(prev.map(t => t.hash))
          return [...prev, ...records.filter((t: TransactionRecord) => !seen.has(t.hash))]
        })
      } else {
        setTransactions(records)
      }
      setHasMore(result.hasMore)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsLoading(false)
    }
  }, [])

  const handleSearch = useCallback(() => {
    setSubmittedAddress(address)
    setPage(1)
    setTransactions([])
  }, [address])

  useEffect(() => {
    if (submittedAddress) {
      fetchHistory(submittedAddress, selectedChain, 1, false)
    }
  }, [submittedAddress, selectedChain, fetchHistory])

  const handleLoadMore = () => {
    const nextPage = page + 1
    setPage(nextPage)
    fetchHistory(submittedAddress, selectedChain, nextPage, true)
  }

  const handleRefresh = () => {
    setPage(1)
    fetchHistory(submittedAddress, selectedChain, 1, false)
  }

  const allTransactions = [
    ...transactions,
    ...pendingTransactions.filter(
      p => p.chainId === selectedChain && p.status === "pending"
    ),
    ...pendingTxsByChain.map((tx): TransactionRecord => ({
      hash: tx.hash,
      blockNumber: 0,
      blockHash: '',
      timestamp: tx.timestamp,
      from: tx.from,
      to: tx.to,
      value: tx.amount,
      valueFormatted: tx.amount,
      gasUsed: '0',
      gasPrice: '0',
      fee: '0',
      feeFormatted: '0',
      status: tx.status,
      chainId: tx.chainId,
    })),
  ]

  return (
    <div className="bg-vault-card rounded-xl p-4 border border-vault-border">
      <h2 className="text-lg font-semibold text-vault-text mb-4">Transaction History</h2>

      {/* Controls */}
      <div className="flex flex-col sm:flex-row gap-3 mb-4">
        <input
          type="text"
          placeholder="Enter wallet address (0x...)"
          value={address}
          onChange={e => setAddress(e.target.value)}
          className="flex-1 bg-vault-bg border border-vault-border rounded-lg px-4 py-2.5 text-vault-text placeholder:text-gray-500 focus:border-transparent focus:ring-2 focus:ring-vault-gradient/50 font-mono text-sm"
        />
        <select
          value={selectedChain}
          onChange={e => setSelectedChain(Number(e.target.value))}
          className="bg-vault-bg border border-vault-border rounded-lg px-4 py-2.5 text-vault-text focus:border-transparent focus:ring-2 focus:ring-vault-gradient/50 text-sm min-w-[140px]"
        >
          {CHAINS.map(c => (
            <option key={c.id} value={c.id}>{c.name}</option>
          ))}
        </select>
        <button
          onClick={handleSearch}
          disabled={!address || isLoading}
          className="px-5 py-2.5 bg-vault-gradient text-white rounded-lg font-medium hover:opacity-90 transition disabled:opacity-50 text-sm whitespace-nowrap"
        >
          Search
        </button>
        {submittedAddress && (
          <button
            onClick={handleRefresh}
            disabled={isLoading}
            className="px-4 py-2.5 bg-vault-border text-vault-text-secondary rounded-lg hover:bg-vault-bg transition disabled:opacity-50 text-sm"
          >
            {isLoading ? "Loading..." : "↻ Refresh"}
          </button>
        )}
      </div>

      {/* Error */}
      {error && (
        <div className="mb-3 p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 text-sm">
          {error}
        </div>
      )}

      {/* Loading state */}
      {isLoading && transactions.length === 0 && (
        <div className="flex justify-center py-8">
          <div className="animate-spin h-6 w-6 border-2 border-vault-gradient border-t-transparent rounded-full" />
        </div>
      )}

      {/* Empty state */}
      {!isLoading && submittedAddress && transactions.length === 0 && !error && (
        <div className="text-center py-8 text-gray-500 text-sm">
          <p>No transactions found for this address.</p>
          <p className="mt-1">Make sure the address is correct for the selected chain.</p>
        </div>
      )}

      {/* No search yet */}
      {!submittedAddress && !isLoading && (
        <div className="text-center py-8 text-gray-500 text-sm">
          <p>Enter a wallet address above to view transaction history.</p>
        </div>
      )}

      {/* Transaction list */}
      {transactions.length > 0 && (
        <div className="space-y-2">
          {allTransactions.map(tx => (
            <div
              key={tx.hash}
              className="bg-vault-bg rounded-lg p-3 border border-vault-border/50 hover:border-vault-border transition"
            >
              <div className="flex justify-between items-start mb-2">
                <div className="flex items-center gap-2">
                  <span className="text-xs text-gray-400 font-mono">
                    {shortenAddress(tx.from)}
                  </span>
                  <span className="text-gray-500 text-xs">→</span>
                  <span className="text-xs text-gray-400 font-mono">
                    {shortenAddress(tx.to || "")}
                  </span>
                </div>
                <TransactionStatusBadge status={tx.status} size="sm" />
              </div>
              <div className="flex justify-between items-center">
                <span className="text-sm font-medium text-vault-text">
                  {formatValue(tx.valueFormatted || "0", symbol)}
                </span>
                <div className="text-right">
                  <span className="text-xs text-gray-500">
                    {formatTimestamp(tx.timestamp)}
                  </span>
                  {tx.feeFormatted && (
                    <span className="block text-xs text-gray-600">
                      Fee: {tx.feeFormatted} {symbol}
                    </span>
                  )}
                </div>
              </div>
              <div className="mt-1.5">
                <span className="text-xs text-gray-600 font-mono">
                  {tx.hash.slice(0, 10)}...{tx.hash.slice(-8)}
                </span>
              </div>
            </div>
          ))}

          {/* Load more */}
          {hasMore && (
            <button
              onClick={handleLoadMore}
              disabled={isLoading}
              className="w-full py-2.5 bg-vault-border/50 text-vault-text-secondary rounded-lg hover:bg-vault-border transition text-sm disabled:opacity-50"
            >
              {isLoading ? "Loading..." : "Load More"}
            </button>
          )}
        </div>
      )}
    </div>
  )
}

export default TransactionHistory

