export interface NFT {
  tokenId: string
  contractAddress: string
  contractType: 'erc721' | 'erc1155'
  name: string
  symbol: string
  description?: string
  imageUrl?: string
  animationUrl?: string
  externalUrl?: string
  attributes?: NFTAttribute[]
  balance?: string // For ERC-1155
  chainId: number
  owner: string
}

export interface NFTAttribute {
  traitType: string
  value: string | number
  displayType?: string
}

export interface NFTCollection {
  contractAddress: string
  name: string
  symbol: string
  contractType: 'erc721' | 'erc1155'
  totalSupply: string
  nfts: NFT[]
  chainId: number
}

export interface NFTTransferParams {
  chainId: number
  contractAddress: string
  from: string
  to: string
  tokenId: string
  amount?: string // For ERC-1155, defaults to "1"
}

export interface NFTBalance {
  contractAddress: string
  tokenId: string
  balance: string
  contractType: 'erc721' | 'erc1155'
  chainId: number
}