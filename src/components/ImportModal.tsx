import { useState, useRef } from 'react'
import { importFromKeystore, importFromMnemonic, validateMnemonic, validatePrivateKey, readFileAsText } from '../utils/walletExport'
import { validateKeystore } from '../utils/walletExport'
import { useWalletStore } from '../stores/walletStore'
import './ImportModal.css'

interface ImportModalProps {
  onClose: () => void
  onSuccess: (wallet: { address: string; name: string }) => void
}

type ImportFormat = 'keystore' | 'mnemonic' | 'privatekey'

export function ImportModal({ onClose, onSuccess }: ImportModalProps) {
  const [format, setFormat] = useState<ImportFormat>('keystore')
  const [data, setData] = useState('')
  const [password, setPassword] = useState('')
  const [walletName, setWalletName] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)
  
  const fileInputRef = useRef<HTMLInputElement>(null)
  const importWallet = useWalletStore(state => state.importWallet)
  
  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return
    
    try {
      const content = await readFileAsText(file)
      setData(content)
    } catch {
      setError('Failed to read file')
    }
  }
  
  const handleImport = async () => {
    setError('')
    
    if (!data) {
      setError('Please provide import data')
      return
    }
    
    if (!password) {
      setError('Password is required for encryption')
      return
    }
    
    if (!walletName.trim()) {
      setError('Please provide a wallet name')
      return
    }
    
    setLoading(true)
    
    try {
      if (format === 'keystore') {
        // Validate keystore JSON
        try {
          const parsed = JSON.parse(data)
          if (!validateKeystore(parsed)) {
            throw new Error('Invalid keystore format')
          }
        } catch {
          setError('Invalid keystore JSON format')
          setLoading(false)
          return
        }
        
        const result = await importFromKeystore(data, password)
        await importWallet(walletName.trim(), password, result.privateKey, 'privateKey')
        onSuccess({
          address: result.address,
          name: walletName.trim()
        })
      } else if (format === 'mnemonic') {
        const cleanedData = data.trim()
        if (!validateMnemonic(cleanedData)) {
          setError('Invalid mnemonic phrase. Must be 12 or 24 words.')
          setLoading(false)
          return
        }
        
        const result = await importFromMnemonic(cleanedData, password)
        await importWallet(walletName.trim(), password, cleanedData, 'mnemonic')
        onSuccess({
          address: result.address,
          name: walletName.trim()
        })
      } else {
        const cleanedData = data.trim().replace(/^0x/, '')
        if (!validatePrivateKey(cleanedData)) {
          setError('Invalid private key format. Must be 64 hex characters.')
          setLoading(false)
          return
        }
        
        await importWallet(walletName.trim(), password, cleanedData, 'privateKey')
        // Derive address from private key for callback
        const { derivePublicKey } = await import('../utils/crypto')
        const publicKey = await derivePublicKey(cleanedData)
        const { publicKeyToAddress } = await import('../utils/crypto')
        const address = await publicKeyToAddress(publicKey)
        onSuccess({
          address,
          name: walletName.trim()
        })
      }
      
      onClose()
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Import failed')
    } finally {
      setLoading(false)
    }
  }
  
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <h2>Import Wallet</h2>
        
        <div className="format-selector">
          <label>
            <input
              type="radio"
              name="importFormat"
              value="keystore"
              checked={format === 'keystore'}
              onChange={() => setFormat('keystore')}
            />
            Keystore File
          </label>
          <label>
            <input
              type="radio"
              name="importFormat"
              value="mnemonic"
              checked={format === 'mnemonic'}
              onChange={() => setFormat('mnemonic')}
            />
            Mnemonic Phrase
          </label>
          <label>
            <input
              type="radio"
              name="importFormat"
              value="privatekey"
              checked={format === 'privatekey'}
              onChange={() => setFormat('privatekey')}
            />
            Private Key
          </label>
        </div>
        
        <div className="form-group">
          <label htmlFor="walletName">Wallet Name</label>
          <input
            type="text"
            id="walletName"
            value={walletName}
            onChange={e => setWalletName(e.target.value)}
            placeholder="My Wallet"
          />
        </div>
        
        <div className="form-group">
          <label htmlFor="data">
            {format === 'keystore' ? 'Keystore File' : 
             format === 'mnemonic' ? 'Mnemonic Phrase' : 'Private Key'}
          </label>
          {format === 'keystore' ? (
            <div className="file-input-wrapper">
              <input
                type="file"
                ref={fileInputRef}
                onChange={handleFileUpload}
                accept=".json"
              />
              {data && <span className="file-selected">File selected</span>}
            </div>
          ) : (
            <textarea
              id="data"
              value={data}
              onChange={e => setData(e.target.value)}
              placeholder={
                format === 'mnemonic'
                  ? 'word1 word2 word3 ...'
                  : '0x...'
              }
              rows={format === 'mnemonic' ? 3 : 2}
            />
          )}
        </div>
        
        <div className="form-group">
          <label htmlFor="importPassword">Encryption Password</label>
          <input
            type="password"
            id="importPassword"
            value={password}
            onChange={e => setPassword(e.target.value)}
            placeholder="Password to encrypt this wallet locally"
          />
          <span className="hint">This password will encrypt your wallet data locally</span>
        </div>
        
        {error && <p className="error">{error}</p>}
        
        <div className="button-group">
          <button onClick={handleImport} disabled={loading} className="btn-primary">
            {loading ? 'Importing...' : 'Import'}
          </button>
          <button onClick={onClose} className="btn-secondary">
            Cancel
          </button>
        </div>
      </div>
    </div>
  )
}
