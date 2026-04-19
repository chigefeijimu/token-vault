export type TxStatus = "pending" | "confirmed" | "failed"
export interface TransactionRecord {
  hash: string
  blockNumber: number
  blockHash: string
  timestamp: number
  from: string
  to: string
  value: string
  valueFormatted: string
  gasUsed: string
  gasPrice: string
  fee: string
  feeFormatted: string
  feeUsd?: number
  status: TxStatus
  chainId: number
  isErc20?: boolean
  tokenSymbol?: string
  tokenDecimals?: number
  tokenLogoUrl?: string
}
export interface TransactionHistoryFilter {
  address: string
  chainId: number
  startBlock?: number
  endBlock?: number
  status?: TxStatus
  tokenAddress?: string
  page?: number
  pageSize?: number
}
export interface TransactionHistoryResult {
  transactions: TransactionRecord[]
  totalCount: number
  page: number
  pageSize: number
  hasMore: boolean
}
export interface TransactionReceiptRust {
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
}

// Convert Rust transaction receipt to TransactionRecord
export function parseRustReceipt(receipt: TransactionReceiptRust, chainId: number): TransactionRecord {
  const status = receipt.status === "0x1" || receipt.status === "1" ? "confirmed" : "failed"
  const gasUsed = BigInt(receipt.gas_used)
  const gasPrice = BigInt(receipt.gas_price)
  const fee = gasUsed * gasPrice
  const feeFormatted = (Number(fee) / 1e18).toFixed(8)
  const valueFormatted = (Number(BigInt(receipt.value)) / 1e18).toFixed(8)
  
  return {
    hash: receipt.transaction_hash,
    blockNumber: receipt.block_number,
    blockHash: receipt.block_hash,
    timestamp: receipt.timestamp,
    from: receipt.from,
    to: receipt.to || '',
    value: receipt.value,
    valueFormatted,
    gasUsed: receipt.gas_used,
    gasPrice: receipt.gas_price,
    fee: fee.toString(),
    feeFormatted,
    status: status as TxStatus,
    chainId,
  }
}
