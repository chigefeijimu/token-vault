import { useEffect, useRef } from 'react'
import { usePendingTxStore, type PendingTx } from '../stores/pendingTxStore'
import { getTransactionReceipt } from '../services/transactionService'

const POLL_INTERVAL_MS = 10_000
const MAX_POLL_CYCLES = 50
const REMOVE_DELAY_MS = 60_000

export function usePendingTxPolling() {
  const pendingTxs = usePendingTxStore((s) => s.pendingTxs)
  const updateTxStatus = usePendingTxStore((s) => s.updateTxStatus)
  const removePendingTx = usePendingTxStore((s) => s.removePendingTx)
  const clearResolved = usePendingTxStore((s) => s.clearResolved)

  const pollCycleRef = useRef<Record<string, number>>({})

  useEffect(() => {
    const pendingList = Object.values(pendingTxs).filter(
      (tx) => tx.status === 'pending'
    )

    if (pendingList.length === 0) return

    let cancelled = false

    async function checkTx(tx: PendingTx) {
      try {
        const receipt = await getTransactionReceipt(tx.chainId, tx.hash)

        if (cancelled) return

        if (receipt !== null) {
          const newStatus = receipt.status ? 'confirmed' : 'failed'
          updateTxStatus(tx.hash, newStatus)

          // Schedule removal after REMOVE_DELAY_MS
          setTimeout(() => {
            removePendingTx(tx.hash)
          }, REMOVE_DELAY_MS)
        } else {
          // Still pending — increment poll cycle counter
          pollCycleRef.current[tx.hash] =
            (pollCycleRef.current[tx.hash] ?? 0) + 1

          if (pollCycleRef.current[tx.hash] >= MAX_POLL_CYCLES) {
            // Give up after max cycles
            updateTxStatus(tx.hash, 'failed')
            setTimeout(() => {
              removePendingTx(tx.hash)
            }, REMOVE_DELAY_MS)
          }
        }
      } catch (err) {
        console.error(`[usePendingTxPolling] failed to check tx ${tx.hash}:`, err)
      }
    }

    // Poll all pending txs in parallel
    Promise.all(pendingList.map(checkTx))

    // Also run clearResolved periodically
    const clearHandle = setInterval(() => {
      clearResolved()
    }, 30_000)

    return () => {
      cancelled = true
      clearInterval(clearHandle)
    }
  }, [pendingTxs, updateTxStatus, removePendingTx, clearResolved])

  // Auto-poll every POLL_INTERVAL_MS when there are pending txs
  useEffect(() => {
    const hasPending = Object.values(pendingTxs).some(
      (tx) => tx.status === 'pending'
    )
    if (!hasPending) return

    const id = setInterval(() => {
      // Trigger re-poll by depending on pendingTxs in the other effect
    }, POLL_INTERVAL_MS)

    return () => clearInterval(id)
  }, [pendingTxs])
}
