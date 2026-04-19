export interface GasPrice {
  low: string
  medium: string
  high: string
  lowTime: number
  mediumTime: number
  highTime: number
  baseFee?: string
  congestion: number
}

export interface GasEstimate {
  gasLimit: string
  gasPrice: string
  maxPriorityFee?: string
  maxFee?: string
  totalFee: string
  totalFeeUsd?: number
  isEip1559: boolean
}

export interface GasOptimization {
  currentGasPrice: string
  suggestedGasPrice: string
  savingsPercent: number
  savingsAmount: string
  reason: string
  recommendedTiming: GasTiming
  alternativeChains: AlternativeChain[]
}

export interface GasTiming {
  immediate: boolean
  waitSeconds?: number
  bestWindowStart?: number
  bestWindowEnd?: number
}

export interface AlternativeChain {
  chainId: number
  chainName: string
  gasPrice: string
  savingsPercent: number
}

export interface GasInfo {
  chainId: number
  gasPrices: GasPrice
  estimates?: GasEstimate
  optimization?: GasOptimization
  timestamp: number
  source: string
}

export interface FeeBreakdown {
  baseFee: string
  priorityFee: string
  gasLimit: string
  gasPrice: string
  totalNative: string
  totalUsd?: number
  currencySymbol: string
}

export type GasPriority = 'slow' | 'standard' | 'fast'