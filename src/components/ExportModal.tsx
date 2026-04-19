import { useState } from 'react'
import { exportMnemonic, exportKeystoreFile, downloadFile } from '../utils/walletExport'
import { useWalletStore } from '../stores/walletStore'
import './ExportModal.css'

interface ExportModalProps {
  walletId: string
  onClose: () => void
}

type ExportFormat = 'mnemonic' | 'keystore'

export function ExportModal({ walletId, onClose }: ExportModalProps) {
  const [format, setFormat] = useState<ExportFormat>('mnemonic')
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)
  const [exportedData, setExportedData] = useState<string | null>(null)
  
  const wallet = useWalletStore(state => state.wallets.find(w => w.id === walletId))
  
  const handleExport = async () => {
    if (!wallet) return
    
    setError('')
    
    if (!password) {
      setError('Password is required')
      return
    }
    
    if (format === 'mnemonic') {
      if (password !== confirmPassword) {
        setError('Passwords do not match')
        return
      }
      if (!wallet.encryptedMnemonic) {
        setError('Wallet does not have a mnemonic to export')
        return
      }
    }
    
    setLoading(true)
    
    try {
      if (format === 'mnemonic') {
        const mnemonic = await exportMnemonic(wallet.encryptedMnemonic!, password)
        setExportedData(mnemonic)
      } else if (format === 'keystore') {
        const { encryptedPrivateKey } = wallet
        if (!encryptedPrivateKey) {
          setError('Wallet does not have a private key to export')
          setLoading(false)
          return
        }
        // Decrypt private key first, then export as keystore
        const { decryptData } = await import('../utils/crypto')
        const privateKey = await decryptData(encryptedPrivateKey, password)
        const { content } = await exportKeystoreFile(privateKey, password, wallet.address)
        setExportedData(content)
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Export failed')
    } finally {
      setLoading(false)
    }
  }
  
  const handleDownload = () => {
    if (!exportedData) return
    
    if (format === 'mnemonic') {
      downloadFile(`mnemonic-${wallet?.address.slice(0, 8)}.txt`, exportedData, 'text/plain')
    } else {
      downloadFile(`UTC--${new Date().toISOString().replace(/[:.]/g, '-')}--${wallet?.address.toLowerCase()}.json`, exportedData)
    }
  }
  
  const handleCopyToClipboard = () => {
    if (!exportedData) return
    navigator.clipboard.writeText(exportedData)
  }
  
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={e => e.stopPropagation()}>
        <h2>Export Wallet</h2>
        
        {!wallet && <p className="error">Wallet not found</p>}
        
        {wallet && (
          <>
            <div className="wallet-info">
              <span className="label">Exporting:</span>
              <span className="address">{wallet.name} ({wallet.address.slice(0, 8)}...)</span>
            </div>
            
            <div className="format-selector">
              <label>
                <input
                  type="radio"
                  name="format"
                  value="mnemonic"
                  checked={format === 'mnemonic'}
                  onChange={() => setFormat('mnemonic')}
                />
                Mnemonic Phrase
              </label>
              <label>
                <input
                  type="radio"
                  name="format"
                  value="keystore"
                  checked={format === 'keystore'}
                  onChange={() => setFormat('keystore')}
                />
                Keystore File (JSON)
              </label>
            </div>
            
            <div className="form-group">
              <label htmlFor="password">Password (for encryption)</label>
              <input
                type="password"
                id="password"
                value={password}
                onChange={e => setPassword(e.target.value)}
                placeholder="Enter password"
              />
            </div>
            
            {format === 'mnemonic' && (
              <div className="form-group">
                <label htmlFor="confirmPassword">Confirm Password</label>
                <input
                  type="password"
                  id="confirmPassword"
                  value={confirmPassword}
                  onChange={e => setConfirmPassword(e.target.value)}
                  placeholder="Confirm password"
                />
              </div>
            )}
            
            {error && <p className="error">{error}</p>}
            
            {!exportedData && (
              <div className="warning-box">
                <strong>⚠️ Warning:</strong> Exporting your private key or mnemonic gives anyone 
                access to your funds. Never share these with anyone.
              </div>
            )}
            
            {exportedData ? (
              <div className="export-result">
                <div className="success-message">Export successful!</div>
                <div className="exported-data">
                  {format === 'mnemonic' ? (
                    <div className="mnemonic-display">
                      {exportedData.split(' ').map((word, i) => (
                        <span key={i} className="word">{i + 1}. {word}</span>
                      ))}
                    </div>
                  ) : (
                    <pre className="keystore-preview">{exportedData.slice(0, 200)}...</pre>
                  )}
                </div>
                <div className="button-group">
                  <button onClick={handleDownload} className="btn-primary">
                    Download {format === 'mnemonic' ? 'Text File' : 'JSON'}
                  </button>
                  {format === 'mnemonic' && (
                    <button onClick={handleCopyToClipboard} className="btn-secondary">
                      Copy to Clipboard
                    </button>
                  )}
                </div>
              </div>
            ) : (
              <div className="button-group">
                <button onClick={handleExport} disabled={loading} className="btn-primary">
                  {loading ? 'Exporting...' : 'Export'}
                </button>
                <button onClick={onClose} className="btn-secondary">
                  Cancel
                </button>
              </div>
            )}
          </>
        )}
      </div>
    </div>
  )
}