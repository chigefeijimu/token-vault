import React, { useState, useCallback } from "react"
import { useNFTs } from "../hooks/useNFTs"
import { CHAINS } from "../types/wallet"
import type { NFT } from "../types/nft"

interface NFTGalleryProps {
  address?: string
  chainId?: number
}

function NFTCard({ nft }: { nft: NFT }) {
  const [loaded, setLoaded] = useState(false)
  const [error, setError] = useState(false)

  const isERC1155 = nft.contractType === "erc1155"

  return (
    <div className="bg-vault-card rounded-xl border border-vault-border overflow-hidden hover:border-vault-border/80 transition group">
      <div className="relative aspect-square bg-vault-bg overflow-hidden">
        {!error && nft.imageUrl ? (
          <>
            {!loaded && (
              <div className="absolute inset-0 bg-vault-border animate-pulse" />
            )}
            <img
              src={nft.imageUrl}
              alt={nft.name || nft.tokenId}
              className={`w-full h-full object-cover transition-opacity duration-300 ${loaded ? "opacity-100" : "opacity-0"}`}
              onLoad={() => setLoaded(true)}
              onError={() => setError(true)}
              loading="lazy"
            />
          </>
        ) : (
          <div className="absolute inset-0 flex items-center justify-center">
            <span className="text-vault-border text-4xl opacity-30 select-none">◇</span>
          </div>
        )}
      </div>
      <div className="p-3">
        <p className="text-xs text-gray-500 truncate mb-0.5">{nft.symbol || nft.contractAddress.slice(0, 10)}</p>
        <p className="text-sm font-medium text-vault-text truncate mb-2" title={nft.name || `#${nft.tokenId}`}>
          {nft.name || `#${nft.tokenId}`}
        </p>
        <span
          className={`inline-block text-xs font-semibold px-2 py-0.5 rounded-full ${
            isERC1155
              ? "bg-purple-500/20 text-purple-400"
              : "bg-blue-500/20 text-blue-400"
          }`}
        >
          {isERC1155 ? "ERC-1155" : "ERC-721"}
        </span>
      </div>
    </div>
  )
}

function ShimmerGrid({ count = 12 }: { count?: number }) {
  return (
    <div className="grid gap-4" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(160px, 1fr))" }}>
      {Array.from({ length: count }).map((_, i) => (
        <div key={i} className="rounded-xl border border-vault-border overflow-hidden">
          <div className="aspect-square bg-vault-border animate-pulse" />
          <div className="p-3 space-y-2">
            <div className="h-3 bg-vault-border rounded animate-pulse w-3/4" />
            <div className="h-4 bg-vault-border rounded animate-pulse w-full" />
            <div className="h-5 bg-vault-border rounded animate-pulse w-1/2" />
          </div>
        </div>
      ))}
    </div>
  )
}

export const NFTGallery: React.FC<NFTGalleryProps> = ({
  address: initialAddress = "",
  chainId: initialChainId = 1,
}) => {
  const [address, setAddress] = useState(initialAddress)
  const [selectedChain, setSelectedChain] = useState(initialChainId)
  const [submittedAddress, setSubmittedAddress] = useState(initialAddress)
  const [selectedCollection, setSelectedCollection] = useState<string>("")
  const [selectedTokenType, setSelectedTokenType] = useState<string>("")

  const { nfts, collections, isLoading, error } = useNFTs(
    submittedAddress,
    { chainId: selectedChain }
  )

  const handleSearch = useCallback(() => {
    setSubmittedAddress(address)
  }, [address])

  const handleRefresh = () => {
    setSubmittedAddress(address)
  }

  const collectionOptions = [
    { value: "", label: "All Collections" },
    ...collections.map(c => ({
      value: c.contractAddress,
      label: c.name || c.symbol || c.contractAddress.slice(0, 10),
    })),
  ]

  const tokenTypeOptions = [
    { value: "", label: "All Types" },
    { value: "erc721", label: "ERC-721" },
    { value: "erc1155", label: "ERC-1155" },
  ]

  const filteredNFTs = nfts.filter(nft => {
    if (selectedCollection && nft.contractAddress !== selectedCollection) return false
    if (selectedTokenType && nft.contractType !== selectedTokenType) return false
    return true
  })

  return (
    <div className="bg-vault-card rounded-xl p-4 border border-vault-border">
      <h2 className="text-lg font-semibold text-vault-text mb-4">NFT Gallery</h2>

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

      {submittedAddress && (
        <div className="flex flex-wrap gap-2 mb-4">
          <select
            value={selectedCollection}
            onChange={e => setSelectedCollection(e.target.value)}
            className="bg-vault-bg border border-vault-border rounded-lg px-3 py-2 text-vault-text text-sm focus:border-transparent focus:ring-2 focus:ring-vault-gradient/50 min-w-[160px]"
          >
            {collectionOptions.map(o => (
              <option key={o.value} value={o.value}>{o.label}</option>
            ))}
          </select>
          <select
            value={selectedTokenType}
            onChange={e => setSelectedTokenType(e.target.value)}
            className="bg-vault-bg border border-vault-border rounded-lg px-3 py-2 text-vault-text text-sm focus:border-transparent focus:ring-2 focus:ring-vault-gradient/50 min-w-[140px]"
          >
            {tokenTypeOptions.map(o => (
              <option key={o.value} value={o.value}>{o.label}</option>
            ))}
          </select>
        </div>
      )}

      {error && (
        <div className="mb-3 p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 text-sm">
          {error}
        </div>
      )}

      {isLoading && <ShimmerGrid />}

      {!isLoading && !submittedAddress && (
        <div className="text-center py-12 text-gray-500 text-sm">
          <p>Enter a wallet address above to view NFTs.</p>
        </div>
      )}

      {!isLoading && submittedAddress && filteredNFTs.length === 0 && !error && (
        <div className="text-center py-12 text-gray-500 text-sm">
          <p>No NFTs found for this address{selectedTokenType ? ` (${selectedTokenType.toUpperCase()})` : ""}.</p>
        </div>
      )}

      {!isLoading && filteredNFTs.length > 0 && (
        <div className="grid gap-4" style={{ gridTemplateColumns: "repeat(auto-fill, minmax(160px, 1fr))" }}>
          {filteredNFTs.map(nft => (
            <NFTCard key={`${nft.contractAddress}-${nft.tokenId}`} nft={nft} />
          ))}
        </div>
      )}
    </div>
  )
}

export default NFTGallery
