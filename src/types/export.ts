// Wallet export/import type definitions

export interface KeystoreExport {
  version: number
  id: string
  crypto: {
    ciphertext: string
    cipher: string
    cipherparams: {
      iv: string
    }
    kdf: string
    kdfparams: {
      dklen: number
      salt: string
      p: number
      prf: string
    }
    mac: string
  }
  address: string
}

export interface ImportResult {
  privateKey: string
  address: string
}

export interface WalletExportOptions {
  format: 'keystore' | 'mnemonic' | 'privatekey'
  password: string
  walletId: string
}

export interface WalletImportOptions {
  format: 'keystore' | 'mnemonic' | 'privatekey'
  data: string
  password?: string
  name?: string
}

export interface ExportedWallet {
  format: 'keystore' | 'mnemonic' | 'privatekey'
  data: string | KeystoreExport
  address: string
  chainIds: number[]
}