import { invoke } from '@tauri-apps/api/core'
import { CryptoError } from '../errors'

/**
 * Encrypt data with password
 */
export async function encryptData(data: string, password: string): Promise<string> {
  try {
    return await invoke<string>('encrypt_data', { data, password })
  } catch (e) {
    throw new CryptoError(`Encryption failed: ${e}`)
  }
}

/**
 * Decrypt data with password
 */
export async function decryptData(encrypted: string, password: string): Promise<string> {
  try {
    return await invoke<string>('decrypt_data', { encrypted, password })
  } catch (e) {
    throw new CryptoError(`Decryption failed: ${e}`)
  }
}

/**
 * Generate a random password
 */
export function generatePassword(length: number = 32): string {
  const chars = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*'
  const array = new Uint8Array(length)
  crypto.getRandomValues(array)
  return Array.from(array, byte => chars[byte % chars.length]).join('')
}

/**
 * Hash password for verification
 */
export async function hashPassword(password: string): Promise<string> {
  const encoder = new TextEncoder()
  const data = encoder.encode(password)
  const hashBuffer = await crypto.subtle.digest('SHA-256', data)
  const hashArray = Array.from(new Uint8Array(hashBuffer))
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('')
}