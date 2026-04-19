import { invoke } from '@tauri-apps/api/core'
import type { KeystoreExport, ImportResult, ExportedWallet } from '../types/export'

/**
 * Export wallet as keystore (JSON format)
 */
export async function exportAsKeystore(
  privateKey: string,
  password: string
): Promise<KeystoreExport> {
  const keystoreJson = await invoke<string>('export_keystore', {
    privateKey,
    password
  })
  return JSON.parse(keystoreJson)
}

/**
 * Export wallet as keystore file content (downloadable)
 */
export async function exportKeystoreFile(
  privateKey: string,
  password: string,
  address: string
): Promise<{ filename: string; content: string }> {
  const keystore = await exportAsKeystore(privateKey, password)
  const filename = `UTC--${new Date().toISOString().replace(/[:.]/g, '-')}--${address.toLowerCase()}`
  return {
    filename: `${filename}.json`,
    content: JSON.stringify(keystore, null, 2)
  }
}

/**
 * Export mnemonic phrase
 */
export async function exportMnemonic(
  encryptedMnemonic: string,
  password: string
): Promise<string> {
  return await invoke<string>('export_mnemonic', {
    encryptedMnemonic,
    password
  })
}

/**
 * Import wallet from keystore JSON
 */
export async function importFromKeystore(
  keystoreJson: string,
  password: string
): Promise<ImportResult> {
  const result = await invoke<{ private_key: string; address: string }>('import_keystore', {
    keystoreJson,
    password
  })
  return {
    privateKey: result.private_key,
    address: result.address
  }
}

/**
 * Import wallet from mnemonic phrase
 */
export async function importFromMnemonic(
  mnemonic: string,
  password: string
): Promise<ImportResult & { encryptedMnemonic: string }> {
  const result = await invoke<{ private_key: string; address: string }>('import_mnemonic', {
    mnemonic,
    password
  })
  // Re-encrypt the mnemonic for storage
  const { encryptData } = await import('./crypto')
  const encryptedMnemonic = await encryptData(mnemonic, password)
  return {
    privateKey: result.private_key,
    address: result.address,
    encryptedMnemonic
  }
}

/**
 * Validate keystore JSON structure
 */
export function validateKeystore(keystore: unknown): keystore is KeystoreExport {
  if (!keystore || typeof keystore !== 'object') return false
  const k = keystore as Record<string, unknown>
  return (
    k.version === 3 &&
    typeof k.id === 'string' &&
    typeof k.crypto === 'object' &&
    typeof k.address === 'string'
  )
}

/**
 * Validate mnemonic phrase (basic check)
 */
export function validateMnemonic(mnemonic: string): boolean {
  const words = mnemonic.trim().split(/\s+/)
  return words.length === 12 || words.length === 24
}

/**
 * Validate private key format
 */
export function validatePrivateKey(privateKey: string): boolean {
  const cleaned = privateKey.replace(/^0x/, '')
  return /^[a-fA-F0-9]{64}$/.test(cleaned)
}

/**
 * Download file helper
 */
export function downloadFile(filename: string, content: string, mimeType: string = 'application/json'): void {
  const blob = new Blob([content], { type: mimeType })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  URL.revokeObjectURL(url)
}

/**
 * Read file helper
 */
export async function readFileAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader()
    reader.onload = () => resolve(reader.result as string)
    reader.onerror = () => reject(reader.error)
    reader.readAsText(file)
  })
}