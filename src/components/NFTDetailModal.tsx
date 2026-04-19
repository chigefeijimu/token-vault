import React, { useState, useEffect } from "react"
import type { NFT, NFTTrait } from "../types/nft"
import { CHAINS } from "../types/wallet"

interface NFTDetailModalProps {
  nft: NFT
  onClose: () => void
}

const RARITY_COLORS: Record<string, string> = {
  background: "bg-purple-500/20 text-purple-400",
  eyes: "bg-blue-500/20 text-blue-400",
  clothes: "bg-green-500/20 text-green-400",
  head: "bg-yellow-500/20 text-yellow-400",
  skin: "bg-orange-500/20 text-orange-400",
  fur: "bg-pink-500/20 text-pink-400",
  default: "bg-vault-border text-vault-text-secondary",
}

function getRarityColor(traitType: string): string {
  const key = traitType.toLowerCase()
  return RARITY_COLORS[key] || RARITY_COLORS.default
}

function truncateAddress(address: string): string {
  return `${address.slice(0, 6)}...${address.slice(-4)}`
}

function copyToClipboard(text: string, setCopied: (v: boolean) => void) {
  navigator.clipboard.writeText(text).then(() => {
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  })
}

function ChainBadge({ chainId }: { chainId: number }) {
  const chain = CHAINS.find(c => c.id === chainId)
  if (!chain) return null
  return (
    <span className="inline-flex items-center gap-1 text-xs font-medium px-2 py-0.5 rounded-full bg-vault-border text-vault-text-secondary">
      {chain.name}
    </span>
  )
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false)
  return (
    <button
      onClick={() => copyToClipboard(text, setCopied)}
      className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded bg-vault-border/60 hover:bg-vault-border text-vault-text-secondary hover:text-vault-text transition"
      title="Copy to clipboard"
    >
      {copied ? (
        <>
          <span>✓ Copied</span>
        </>
      ) : (
        <>
          <span className="opacity-60">0x</span>
          <span>{text.slice(2, 6)}...{text.slice(-4)}</span>
        </>
      )}
    </button>
  )
}

function TraitBadge({ trait }: { trait: NFTTrait }) {
  return (
    <div className="flex flex-col gap-1 p-3 rounded-lg border border-vault-border bg-vault-bg hover:border-vault-border/80 transition">
      <span className="text-xs font-bold text-vault-text-secondary uppercase tracking-wide">
        {trait.traitType}
      </span>
      <span className={`text-sm font-medium px-2 py-0.5 rounded-md w-fit ${getRarityColor(trait.traitType)}`}>
        {trait.displayType ? trait.displayType : String(trait.value)}
      </span>
    </div>
  )
}

export const NFTDetailModal: React.FC<NFTDetailModalProps> = ({ nft, onClose }) => {
  const [lightboxOpen, setLightboxOpen] = useState(false)
  const [imageLoaded, setImageLoaded] = useState(false)
  const [imageError, setImageError] = useState(false)

  const chain = CHAINS.find(c => c.id === nft.chainId)
  const explorerUrl = chain?.explorerUrl ?? "https://etherscan.io"
  const tokenUrl = `${explorerUrl}/token/${nft.contractAddress}?a=${nft.tokenId}`

  const isAnimation = Boolean(nft.animationUrl)
  const isVideo = nft.animationUrl
    ? /\.(mp4|webm|ogg)$/i.test(nft.animationUrl) ||
      nft.animationUrl.includes("video")
    : false
  const isAudio = nft.animationUrl
    ? /\.(mp3|wav|ogg|m4a)$/i.test(nft.animationUrl) ||
      nft.animationUrl.includes("audio") ||
      nft.animationUrl.includes("soundcloud") ||
      nft.animationUrl.includes("spotify")
    : false

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (lightboxOpen) setLightboxOpen(false)
        else onClose()
      }
    }
    document.addEventListener("keydown", handleKey)
    return () => document.removeEventListener("keydown", handleKey)
  }, [lightboxOpen, onClose])

  return (
    <>
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          className="absolute inset-0 bg-black/70 backdrop-blur-sm"
          onClick={onClose}
        />
        <div className="relative z-10 w-full max-w-2xl max-h-[90vh] overflow-y-auto bg-vault-card rounded-2xl border border-vault-border shadow-2xl">
          <button
            onClick={onClose}
            className="absolute top-3 right-3 z-20 w-8 h-8 flex items-center justify-center rounded-full bg-vault-border/80 hover:bg-vault-border text-vault-text-secondary hover:text-vault-text transition"
          >
            ✕
          </button>

          <div className="p-5">
            <div className="relative aspect-square rounded-xl overflow-hidden bg-vault-bg mb-4 cursor-zoom-in"
              onClick={() => nft.imageUrl && !imageError && setLightboxOpen(true)}>
              {!imageLoaded && !imageError && (
                <div className="absolute inset-0 bg-vault-border animate-pulse" />
              )}
              {nft.imageUrl && !imageError ? (
                <img
                  src={nft.imageUrl}
                  alt={nft.name || `#${nft.tokenId}`}
                  className={`w-full h-full object-contain transition-opacity duration-300 ${imageLoaded ? "opacity-100" : "opacity-0"}`}
                  onLoad={() => setImageLoaded(true)}
                  onError={() => { setImageError(true); setImageLoaded(false) }}
                />
              ) : (
                <div className="absolute inset-0 flex items-center justify-center">
                  <span className="text-vault-border text-6xl opacity-30 select-none">◇</span>
                </div>
              )}
              {nft.imageUrl && !imageError && (
                <div className="absolute bottom-2 right-2 text-xs bg-black/50 text-white px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition">
                  🔍 Click to enlarge
                </div>
              )}
            </div>

            <div className="mb-4">
              <div className="flex items-start justify-between gap-2 mb-1">
                <h2 className="text-xl font-semibold text-vault-text break-words">
                  {nft.name || `#${nft.tokenId}`}
                </h2>
                <ChainBadge chainId={nft.chainId} />
              </div>
              {nft.symbol && (
                <p className="text-sm text-vault-text-secondary mb-2">{nft.symbol}</p>
              )}
            </div>

            {nft.description && (
              <p className="text-sm text-gray-400 mb-4 leading-relaxed">{nft.description}</p>
            )}

            <div className="grid grid-cols-2 gap-3 mb-4 text-sm">
              <div className="p-3 rounded-lg border border-vault-border bg-vault-bg">
                <p className="text-xs text-gray-500 mb-1 font-medium">Contract</p>
                <div className="flex items-center gap-1.5 flex-wrap">
                  <span className="text-vault-text font-mono text-xs">
                    {truncateAddress(nft.contractAddress)}
                  </span>
                  <CopyButton text={nft.contractAddress} />
                </div>
              </div>
              <div className="p-3 rounded-lg border border-vault-border bg-vault-bg">
                <p className="text-xs text-gray-500 mb-1 font-medium">Token ID</p>
                <p className="text-vault-text font-mono text-xs break-all">#{nft.tokenId}</p>
              </div>
              <div className="p-3 rounded-lg border border-vault-border bg-vault-bg">
                <p className="text-xs text-gray-500 mb-1 font-medium">Type</p>
                <span className={`inline-block text-xs font-semibold px-2 py-0.5 rounded-full ${
                  nft.contractType === "erc1155"
                    ? "bg-purple-500/20 text-purple-400"
                    : "bg-blue-500/20 text-blue-400"
                }`}>
                  {nft.contractType === "erc1155" ? "ERC-1155" : "ERC-721"}
                </span>
              </div>
              {nft.balance && nft.contractType === "erc1155" && (
                <div className="p-3 rounded-lg border border-vault-border bg-vault-bg">
                  <p className="text-xs text-gray-500 mb-1 font-medium">Balance</p>
                  <p className="text-vault-text font-mono text-sm">{nft.balance}</p>
                </div>
              )}
            </div>

            {isAnimation && (
              <div className="mb-4">
                <p className="text-xs text-gray-500 mb-2 font-medium uppercase tracking-wide">Media</p>
                <div className="rounded-xl overflow-hidden bg-vault-bg border border-vault-border">
                  {isVideo ? (
                    <video
                      src={nft.animationUrl}
                      controls
                      className="w-full max-h-64"
                      playsInline
                    />
                  ) : isAudio ? (
                    <div className="p-4 flex flex-col items-center gap-2">
                      <audio
                        src={nft.animationUrl}
                        controls
                        className="w-full"
                      />
                    </div>
                  ) : (
                    <iframe
                      src={nft.animationUrl}
                      className="w-full h-48"
                      title="NFT Animation"
                      sandbox="allow-scripts allow-same-origin allow-popups"
                    />
                  )}
                </div>
              </div>
            )}

            {nft.attributes && nft.attributes.length > 0 && (
              <div className="mb-4">
                <p className="text-xs text-gray-500 mb-2 font-medium uppercase tracking-wide">
                  Attributes
                </p>
                <div className="grid grid-cols-2 sm:grid-cols-3 gap-2">
                  {nft.attributes.map((trait, i) => (
                    <TraitBadge key={i} trait={trait} />
                  ))}
                </div>
              </div>
            )}

            <div className="flex gap-2">
              <a
                href={tokenUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="flex-1 text-center px-4 py-2.5 bg-vault-gradient text-white rounded-lg font-medium hover:opacity-90 transition text-sm"
              >
                View on Explorer
              </a>
              {nft.externalUrl && (
                <a
                  href={nft.externalUrl}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="px-4 py-2.5 bg-vault-border text-vault-text rounded-lg font-medium hover:bg-vault-bg transition text-sm"
                >
                  Open Original
                </a>
              )}
            </div>
          </div>
        </div>
      </div>

      {lightboxOpen && nft.imageUrl && (
        <div
          className="fixed inset-0 z-[60] flex items-center justify-center bg-black/90"
          onClick={() => setLightboxOpen(false)}
        >
          <button
            className="absolute top-4 right-4 w-10 h-10 flex items-center justify-center rounded-full bg-white/10 hover:bg-white/20 text-white text-xl transition z-10"
            onClick={() => setLightboxOpen(false)}
          >
            ✕
          </button>
          <img
            src={nft.imageUrl}
            alt={nft.name || `#${nft.tokenId}`}
            className="max-w-[90vw] max-h-[90vh] object-contain rounded-lg"
            onClick={e => e.stopPropagation()}
          />
        </div>
      )}
    </>
  )
}

export default NFTDetailModal
