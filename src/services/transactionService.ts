import { invoke } from '@tauri-apps/api/core';

/**
 * Transaction request interface matching the Rust backend
 */
export interface TransactionRequest {
  from: string;
  to: string;
  value: string;
  data?: string;
  gas?: string;
  gasPrice?: string;
  maxFeePerGas?: string;
  maxPriorityFeePerGas?: string;
  nonce?: number;
  chainId: number;
}

/**
 * Send transaction result
 */
export interface SendTransactionResult {
  hash: string;
  nonce: number;
  chainId: number;
}

/**
 * Transaction receipt interface
 */
export interface TransactionReceipt {
  transactionHash: string;
  blockNumber: string;
  blockHash: string;
  from: string;
  to?: string;
  cumulativeGasUsed: string;
  gasUsed: string;
  effectiveGasPrice: string;
  logs: TransactionLog[];
  status: boolean;
}

/**
 * Transaction log interface
 */
export interface TransactionLog {
  address: string;
  topics: string[];
  data: string;
  blockNumber: string;
  transactionHash: string;
  logIndex: number;
  removed: boolean;
}

/**
 * Send a transaction using the wallet's private key
 * @param chainId - The chain ID of the network
 * @param fromAddress - The sender address
 * @param toAddress - The recipient address
 * @param value - Amount in wei/smallest unit (hex string)
 * @param data - Optional hex-encoded data
 * @param gas - Optional gas limit (hex string)
 * @param gasPrice - Optional gas price for legacy transactions (hex string)
 * @param maxFeePerGas - Optional max fee per gas for EIP-1559 (hex string)
 * @param maxPriorityFeePerGas - Optional max priority fee for EIP-1559 (hex string)
 * @param walletId - The wallet ID to use for signing
 * @returns Transaction result with hash
 */
export async function sendTransaction(
  chainId: number,
  fromAddress: string,
  toAddress: string,
  value: string,
  data?: string,
  gas?: string,
  gasPrice?: string,
  maxFeePerGas?: string,
  maxPriorityFeePerGas?: string,
  walletId?: string
): Promise<SendTransactionResult> {
  return invoke<SendTransactionResult>('send_transaction', {
    chainId,
    fromAddress,
    toAddress,
    value,
    data: data || null,
    gas: gas || null,
    gasPrice: gasPrice || null,
    maxFeePerGas: maxFeePerGas || null,
    maxPriorityFeePerGas: maxPriorityFeePerGas || null,
    walletId: walletId || ''
  });
}

/**
 * Get transaction receipt by hash
 * @param chainId - The chain ID
 * @param txHash - Transaction hash
 * @returns Transaction receipt or null if pending
 */
export async function getTransactionReceipt(
  chainId: number,
  txHash: string
): Promise<TransactionReceipt | null> {
  return invoke<TransactionReceipt | null>('get_transaction_receipt', {
    chainId,
    txHash
  });
}

/**
 * Get current gas price
 * @param chainId - The chain ID
 * @returns Gas price info
 */
export async function getGasPrice(chainId: number): Promise<{ slow: string; standard: string; fast: string; unit: string }> {
  return invoke<{ slow: string; standard: string; fast: string; unit: string }>('get_gas_price', { chainId });
}

/**
 * Estimate gas for a transaction
 * @param chainId - The chain ID
 * @param from - Sender address
 * @param to - Recipient address
 * @param value - Amount in hex
 * @param data - Optional hex data
 * @returns Estimated gas in hex string
 */
export async function estimateGas(
  chainId: number,
  from: string,
  to: string,
  value: string,
  data?: string
): Promise<string> {
  return invoke<string>('estimate_gas', {
    chainId,
    from,
    to,
    value,
    data: data || null
  });
}

/**
 * Wait for transaction confirmation
 * @param chainId - The chain ID
 * @param txHash - Transaction hash
 * @param intervalMs - Polling interval in milliseconds
 * @param timeoutMs - Maximum wait time in milliseconds
 * @returns Transaction receipt when confirmed
 */
export async function waitForTransaction(
  chainId: number,
  txHash: string,
  intervalMs: number = 2000,
  timeoutMs: number = 120000
): Promise<TransactionReceipt | null> {
  const startTime = Date.now();
  
  while (Date.now() - startTime < timeoutMs) {
    const receipt = await getTransactionReceipt(chainId, txHash);
    
    if (receipt !== null) {
      return receipt;
    }
    
    await new Promise(resolve => setTimeout(resolve, intervalMs));
  }
  
  return null;
}

/**
 * Convert ETH/TOKEN amount to wei/smallest unit
 * @param amount - Amount in ETH/token units
 * @param decimals - Decimals of the token (18 for ETH)
 * @returns Amount in wei/smallest unit as hex string
 */
export function toWei(amount: string | number, decimals: number = 18): string {
  const num = typeof amount === 'string' ? parseFloat(amount) : amount;
  const wei = num * Math.pow(10, decimals);
  return '0x' + Math.floor(wei).toString(16);
}

/**
 * Convert wei/smallest unit to ETH/token amount
 * @param wei - Amount in wei/smallest unit (hex or number)
 * @param decimals - Decimals of the token (18 for ETH)
 * @returns Amount in ETH/token units
 */
export function fromWei(wei: string | number, decimals: number = 18): string {
  const weiStr = typeof wei === 'string' ? wei : wei.toString(16);
  const weiHex = weiStr.startsWith('0x') ? weiStr : '0x' + weiStr;
  const weiBigInt = BigInt(weiHex);
  const divisor = BigInt(10 ** decimals);
  
  const integerPart = weiBigInt / divisor;
  const fractionalPart = weiBigInt % divisor;
  
  const fractionalStr = fractionalPart.toString().padStart(decimals, '0');
  
  return `${integerPart}.${fractionalStr.replace(/0+$/, '')}`;
}