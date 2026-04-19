import { useState, useCallback, useEffect } from "react"
import { invoke } from "@tauri-apps/api/core"
import { CHAINS } from "../types/wallet"
import type { NFT, NFTCollection } from "../types/nft"

const CACHE_TTL_MS = 5 * 60 * 1000 // 5 minutes

interface NFTInfoRust {
  token_id: string
  contract_address: string
  contract_type: string
  name: string
  symbol: string
  description?: string
  image_url?: string
  animation_url?: string
  external_url?: string
  attributes: Array<{ trait_type?: string; value: unknown; display_type?: string }>
  balance?: string
  owner: string
  chain_id: number
}

interface CacheEntry {
  nfts: NFT[]
  collections: NFTCollection[]
  timestamp: number
}

function parseRustNFT(info: NFTInfoRust): NFT {
  return {
    tokenId: info.token_id,
    contractAddress: info.contract_address,
    contractType: info.contract_type as "erc721" | "erc1155",
    name: info.name,
    symbol: info.symbol,
    description: info.description,
    imageUrl: info.image_url,
    animationUrl: info.animation_url,
    externalUrl: info.external_url,
    attributes: (info.attributes ?? []).map(attr => ({
      traitType: attr.trait_type ?? "",
      value: typeof attr.value === "string" ? attr.value : String(attr.value),
      displayType: attr.display_type,
    })),
    balance: info.balance,
    chainId: info.chain_id,
    owner: info.owner,
  }
}

function buildCollections(nfts: NFT[]): NFTCollection[] {
  const map = new Map<string, NFT[]>()
  for (const nft of nfts) {
    const key = nft.contractAddress
    if (!map.has(key)) map.set(key, [])
    map.get(key)!.push(nft)
  }
  return Array.from(map.entries()).map(([contractAddress, group]) => ({
    contractAddress,
    name: group[0]?.name ?? "",
    symbol: group[0]?.symbol ?? "",
    contractType: group[0]?.contractType ?? "erc721",
    totalSupply: String(group.length),
    nfts: group,
    chainId: group[0]?.chainId ?? 1,
  }))
}

function getCacheKey(address: string, chainId: number): string {
  return `nfts:${address.toLowerCase()}:${chainId}`
}

function readCache(address: string, chainId: number): CacheEntry | null {
  try {
    const raw = localStorage.getItem(getCacheKey(address, chainId))
    if (!raw) return null
    const entry: CacheEntry = JSON.parse(raw)
    if (Date.now() - entry.timestamp > CACHE_TTL_MS) {
      localStorage.removeItem(getCacheKey(address, chainId))
      return null
    }
    return entry
  } catch {
    return null
  }
}

function writeCache(address: string, chainId: number, nfts: NFT[], collections: NFTCollection[]): void {
  try {
    const entry: CacheEntry = { nfts, collections, timestamp: Date.now() }
    localStorage.setItem(getCacheKey(address, chainId), JSON.stringify(entry))
  } catch {
    // localStorage may be full or unavailable
  }
}

interface UseNFTsOptions {
  chainId?: number
  contractAddresses?: string[]
  skipCache?: boolean
}

interface UseNFTsResult {
  nfts: NFT[]
  collections: NFTCollection[]
  isLoading: boolean
  error: string | null
  refetch: () => Promise<void>
}

const DEFAULT_CHAIN_ID = 1
const DEFAULT_COUNT = 20

export function useNFTs(address: string, options: UseNFTsOptions = {}): UseNFTsResult {
  const { chainId = DEFAULT_CHAIN_ID, contractAddresses, skipCache = false } = options

  const [nfts, setNfts] = useState<NFT[]>([])
  const [collections, setCollections] = useState<NFTCollection[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const fetch_ = useCallback(async (signal?: AbortSignal) => {
    if (!address || address.length < 20) return

    const chain = CHAINS.find(c => c.id === chainId)
    if (!chain) {
      setError(`Unsupported chain: ${chainId}`)
      return
    }

    setIsLoading(true)
    setError(null)

    try {
      let raw: NFTInfoRust[]

      if (contractAddresses && contractAddresses.length > 0) {
        const results = await Promise.all(
          contractAddresses.map(ca =>
            invoke<NFTInfoRust[]>("get_nfts", {
              rpcUrl: chain.rpcUrl,
              contractAddress: ca,
              owner: address,
              chainId,
              startToken: 0,
              count: DEFAULT_COUNT,
            })
          )
        )
        raw = results.flat()
      } else {
        raw = await invoke<NFTInfoRust[]>("get_nfts", {
          rpcUrl: chain.rpcUrl,
          contractAddress: "",
          owner: address,
          chainId,
          startToken: 0,
          count: DEFAULT_COUNT,
        })
      }

      if (signal?.aborted) return

      const parsed: NFT[] = raw.map(parseRustNFT)
      const grouped: NFTCollection[] = buildCollections(parsed)

      setNfts(parsed)
      setCollections(grouped)
      writeCache(address, chainId, parsed, grouped)
    } catch (err) {
      if (signal?.aborted) return
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      if (!signal?.aborted) setIsLoading(false)
    }
  }, [address, chainId, contractAddresses])

  useEffect(() => {
    if (!address) return

    const cached = !skipCache ? readCache(address, chainId) : null
    if (cached) {
      setNfts(cached.nfts)
      setCollections(cached.collections)
      return
    }

    const controller = new AbortController()
    fetch_(controller.signal)

    return () => controller.abort()
  }, [address, chainId, skipCache]) // eslint-disable-line react-hooks/exhaustive-deps

  const refetch = useCallback(async () => {
    await fetch_()
  }, [fetch_])

  return { nfts, collections, isLoading, error, refetch }
}
