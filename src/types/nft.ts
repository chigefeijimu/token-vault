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
  attributes?: NFTTrait[]
  balance?: string
  chainId: number
  owner: string
}

export interface NFTTrait {
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

export interface NFTFetchOptions {
  chainId: number
  address: string
  contractAddresses?: string[]
  startToken?: number
  count?: number
}
