import { useState, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { SavedAddress, SavedAddressInput, SavedAddressUpdate, AddressValidationResult } from '../types/wallet'

export interface UseSavedAddressesReturn {
  addresses: SavedAddress[]
  loading: boolean
  error: string | null
  fetchAddresses: () => Promise<void>
  fetchByChain: (chainId: number) => Promise<void>
  fetchFavorites: () => Promise<void>
  search: (query: string) => Promise<void>
  add: (input: SavedAddressInput) => Promise<SavedAddress>
  update: (update: SavedAddressUpdate) => Promise<SavedAddress>
  remove: (id: string) => Promise<void>
  toggleFavorite: (id: string) => Promise<SavedAddress>
  exportAddresses: () => Promise<string>
  importAddresses: (json: string, merge: boolean) => Promise<SavedAddress[]>
  validate: (address: string) => Promise<AddressValidationResult>
  clearError: () => void
}

export function useSavedAddresses(): UseSavedAddressesReturn {
  const [addresses, setAddresses] = useState<SavedAddress[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const clearError = useCallback(() => setError(null), [])

  const fetchAddresses = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<SavedAddress[]>('get_saved_addresses')
      setAddresses(result)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  const fetchByChain = useCallback(async (chainId: number) => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<SavedAddress[]>('get_saved_addresses_by_chain', { chainId })
      setAddresses(result)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  const fetchFavorites = useCallback(async () => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<SavedAddress[]>('get_favorite_saved_addresses')
      setAddresses(result)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  const search = useCallback(async (query: string) => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<SavedAddress[]>('search_saved_addresses', { query })
      setAddresses(result)
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }, [])

  const add = useCallback(async (input: SavedAddressInput): Promise<SavedAddress> => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<SavedAddress>('add_saved_address', { input })
      setAddresses(prev => [...prev, result])
      return result
    } catch (err) {
      setError(String(err))
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const update = useCallback(async (update: SavedAddressUpdate): Promise<SavedAddress> => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<SavedAddress>('update_saved_address', { update })
      setAddresses(prev => prev.map(a => a.id === result.id ? result : a))
      return result
    } catch (err) {
      setError(String(err))
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const remove = useCallback(async (id: string) => {
    setLoading(true)
    setError(null)
    try {
      await invoke('delete_saved_address', { id })
      setAddresses(prev => prev.filter(a => a.id !== id))
    } catch (err) {
      setError(String(err))
      throw err
    } finally {
      setLoading(false)
    }
  }, [])

  const toggleFavorite = useCallback(async (id: string): Promise<SavedAddress> => {
    setError(null)
    try {
      const result = await invoke<SavedAddress>('toggle_saved_address_favorite', { id })
      setAddresses(prev => prev.map(a => a.id === result.id ? result : a))
      return result
    } catch (err) {
      setError(String(err))
      throw err
    }
  }, [])

  const exportAddresses = useCallback(async (): Promise<string> => {
    setError(null)
    try {
      return await invoke<string>('export_saved_addresses')
    } catch (err) {
      setError(String(err))
      throw err
    }
  }, [])

  const importAddresses = useCallback(async (json: string, merge: boolean): Promise<SavedAddress[]> => {
    setLoading(true)
    setError(null)
    try {
      const result = await invoke<SavedAddress[]>('import_saved_addresses', { jsonData: json, merge })
      await fetchAddresses()
      return result
    } catch (err) {
      setError(String(err))
      throw err
    } finally {
      setLoading(false)
    }
  }, [fetchAddresses])

  const validate = useCallback(async (address: string): Promise<AddressValidationResult> => {
    setError(null)
    try {
      return await invoke<AddressValidationResult>('validate_address', { address })
    } catch (err) {
      setError(String(err))
      throw err
    }
  }, [])

  return {
    addresses,
    loading,
    error,
    fetchAddresses,
    fetchByChain,
    fetchFavorites,
    search,
    add,
    update,
    remove,
    toggleFavorite,
    exportAddresses,
    importAddresses,
    validate,
    clearError
  }
}
