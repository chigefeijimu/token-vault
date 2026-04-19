import { useState, useEffect, useCallback } from "react"
import { Card, CardHeader, CardTitle, CardContent } from "../components/common/Card"
import { Button } from "../components/common/Button"
import { Input } from "../components/common/Input"
import { CHAINS } from "../types/wallet"
import { useSecurityStore } from "../stores/securityStore"
import { useI18nStore, LANGUAGES as I18N_LANGUAGES } from "../i18n"
import { Lock, Eye, EyeOff, Shield, Clock } from "lucide-react"

const DEFAULT_CHAIN_IDS = [1, 56, 137, 42161, 10, 43114]

const THEMES = [
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
  { value: "system", label: "System" },
]

const CURRENCIES = [
  { value: "USD", label: "USD", symbol: "$" },
  { value: "CNY", label: "CNY", symbol: "¥" },
  { value: "JPY", label: "JPY", symbol: "¥" },
  { value: "EUR", label: "EUR", symbol: "€" },
  { value: "GBP", label: "GBP", symbol: "£" },
]

const STORAGE_KEY = "settings"

export interface RpcConfig {
  chainId: number
  customUrl: string
  enabled: boolean
}

export interface AppSettings {
  language: string
  currency: string
  theme: string
  rpcConfigs: RpcConfig[]
}

const defaultSettings: AppSettings = {
  language: "en",
  currency: "USD",
  theme: "system",
  rpcConfigs: CHAINS.filter(c => DEFAULT_CHAIN_IDS.includes(c.id)).map(chain => ({
    chainId: chain.id,
    customUrl: "",
    enabled: false,
  })),
}

function loadSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return defaultSettings
    const parsed = JSON.parse(raw) as Partial<AppSettings>
    const rpcConfigs = (parsed.rpcConfigs ?? defaultSettings.rpcConfigs).map(rc => {
      const defaultRpc = defaultSettings.rpcConfigs.find(d => d.chainId === rc.chainId)
      return {
        chainId: rc.chainId,
        customUrl: rc.customUrl ?? defaultRpc?.customUrl ?? "",
        enabled: rc.enabled ?? defaultRpc?.enabled ?? false,
      }
    })
    return {
      language: parsed.language ?? defaultSettings.language,
      currency: parsed.currency ?? defaultSettings.currency,
      theme: parsed.theme ?? defaultSettings.theme,
      rpcConfigs,
    }
  } catch {
    return defaultSettings
  }
}

function applyTheme(theme: string) {
  const root = document.documentElement
  if (theme === "system") {
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches
    root.classList.toggle("dark", prefersDark)
  } else {
    root.classList.toggle("dark", theme === "dark")
  }
}

type TestStatus = "idle" | "testing" | "success" | "error"

interface ChainRpcRowProps {
  chainId: number
  config: RpcConfig
  onChange: (updated: RpcConfig) => void
}

function ChainRpcRow({ chainId, config, onChange }: ChainRpcRowProps) {
  const chain = CHAINS.find(c => c.id === chainId)
  const [testStatus, setTestStatus] = useState<TestStatus>("idle")
  const [testMsg, setTestMsg] = useState("")

  const effectiveUrl = config.enabled && config.customUrl ? config.customUrl : (chain?.rpcUrl ?? "")

  const handleTest = useCallback(async () => {
    if (!effectiveUrl) return
    setTestStatus("testing")
    setTestMsg("")
    try {
      const controller = new AbortController()
      const timeout = setTimeout(() => controller.abort(), 8000)
      const res = await fetch(effectiveUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          jsonrpc: "2.0",
          method: "eth_blockNumber",
          params: [],
          id: 1,
        }),
        signal: controller.signal,
      })
      clearTimeout(timeout)
      if (res.ok) {
        const json = await res.json()
        setTestStatus("success")
        setTestMsg(json.result ? `Connected — block: ${parseInt(json.result, 16)}` : "Connected")
      } else {
        setTestStatus("error")
        setTestMsg(`HTTP ${res.status}`)
      }
    } catch (e: unknown) {
      setTestStatus("error")
      if (e instanceof Error && e.name === "AbortError") {
        setTestMsg("Request timed out")
      } else {
        setTestMsg("Connection failed")
      }
    }
  }, [effectiveUrl])

  if (!chain) return null

  return (
    <div className="flex flex-col sm:flex-row sm:items-center gap-2 py-3 border-b border-vault-border last:border-0">
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-vault-text truncate">{chain.name}</p>
        <p className="text-xs text-vault-text-secondary truncate">
          {config.enabled && config.customUrl ? config.customUrl : chain.rpcUrl}
        </p>
      </div>
      <div className="flex items-center gap-2 flex-shrink-0">
        {testStatus !== "idle" && (
          <span
            className={`text-xs px-2 py-0.5 rounded ${
              testStatus === "testing"
                ? "text-vault-text-secondary"
                : testStatus === "success"
                ? "bg-vault-success/20 text-vault-success"
                : "bg-vault-error/20 text-vault-error"
            }`}
          >
            {testStatus === "testing" ? "Testing..." : testMsg}
          </span>
        )}
        <button
          onClick={handleTest}
          disabled={testStatus === "testing"}
          className="px-3 py-1 text-xs rounded border border-vault-border text-vault-text-secondary hover:border-vault-gradient hover:text-vault-text transition disabled:opacity-50"
        >
          Test
        </button>
        <button
          onClick={() => onChange({ ...config, enabled: !config.enabled })}
          className={`relative w-10 h-5 rounded-full transition-colors ${
            config.enabled ? "bg-vault-gradient" : "bg-vault-border"
          }`}
          aria-label={config.enabled ? "Disable custom RPC" : "Enable custom RPC"}
        >
          <span
            className={`absolute top-0.5 w-4 h-4 bg-white rounded-full shadow transition-transform ${
              config.enabled ? "translate-x-5" : "translate-x-0.5"
            }`}
          />
        </button>
      </div>
    </div>
  )
}

// App Lock Settings Component
function AppLockSettings() {
  const hasPasswordSet = useSecurityStore(state => state.hasPasswordSet)
  const lockTimeout = useSecurityStore(state => state.lockTimeout)
  const setLockPassword = useSecurityStore(state => state.setLockPassword)
  const setLockTimeout = useSecurityStore(state => state.setLockTimeout)
  const lock = useSecurityStore(state => state.lock)

  const [showChangePassword, setShowChangePassword] = useState(false)
  const [currentPassword, setCurrentPassword] = useState('')
  const [newPassword, setNewPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [showPassword, setShowPassword] = useState(false)
  const [error, setError] = useState('')
  const [success, setSuccess] = useState(false)

  const verifyLockPassword = useSecurityStore(state => state.verifyLockPassword)

  const timeoutOptions = [
    { value: 1, label: '1 minute' },
    { value: 5, label: '5 minutes' },
    { value: 15, label: '15 minutes' },
    { value: 30, label: '30 minutes' },
  ]

  const handleChangePassword = useCallback(async () => {
    setError('')
    
    if (!currentPassword) {
      setError('Please enter current password')
      return
    }
    if (!newPassword) {
      setError('Please enter new password')
      return
    }
    if (newPassword.length < 4) {
      setError('New password must be at least 4 characters')
      return
    }
    if (newPassword !== confirmPassword) {
      setError('New passwords do not match')
      return
    }

    const isValid = await verifyLockPassword(currentPassword)
    if (!isValid) {
      setError('Current password is incorrect')
      return
    }

    try {
      await setLockPassword(newPassword)
      setSuccess(true)
      setCurrentPassword('')
      setNewPassword('')
      setConfirmPassword('')
      setShowChangePassword(false)
      setTimeout(() => setSuccess(false), 3000)
    } catch {
      setError('Failed to change password')
    }
  }, [currentPassword, newPassword, confirmPassword, verifyLockPassword, setLockPassword])

  const handleDisableLock = useCallback(() => {
    // Clear the lock password by setting a random one that won't be used
    // The app will see hasPasswordSet as false since we clear both
    useSecurityStore.setState({ 
      lockPasswordHash: null, 
      hasPasswordSet: false,
      isLocked: false 
    })
  }, [])

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center gap-2">
          <Lock className="w-5 h-5 text-vault-gradient" />
          <CardTitle>App Lock</CardTitle>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Enable/Disable Toggle */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Shield className="w-4 h-4 text-vault-text-secondary" />
            <span className="text-sm text-vault-text">Password Protection</span>
          </div>
          {hasPasswordSet ? (
            <div className="flex items-center gap-2">
              <span className="text-xs text-green-400">Enabled</span>
              <button
                onClick={handleDisableLock}
                className="px-2 py-1 text-xs rounded border border-red-500/30 text-red-400 hover:bg-red-500/10 transition"
              >
                Disable
              </button>
            </div>
          ) : (
            <span className="text-xs text-gray-500">Not set up</span>
          )}
        </div>

        {/* Auto-lock Timeout */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Clock className="w-4 h-4 text-vault-text-secondary" />
            <span className="text-sm text-vault-text">Auto-lock Timeout</span>
          </div>
          <select
            value={lockTimeout}
            onChange={(e) => setLockTimeout(Number(e.target.value))}
            className="bg-vault-bg border border-vault-border rounded-lg px-3 py-1.5 text-sm text-vault-text focus:outline-none focus:border-vault-gradient"
          >
            {timeoutOptions.map(opt => (
              <option key={opt.value} value={opt.value}>
                {opt.label}
              </option>
            ))}
          </select>
        </div>

        {/* Change Password */}
        {hasPasswordSet && (
          <div className="pt-2 border-t border-vault-border">
            {!showChangePassword ? (
              <button
                onClick={() => setShowChangePassword(true)}
                className="text-sm text-vault-gradient hover:underline"
              >
                Change Password
              </button>
            ) : (
              <div className="space-y-3 pt-2">
                <Input
                  type={showPassword ? 'text' : 'password'}
                  placeholder="Current password"
                  value={currentPassword}
                  onChange={(e) => setCurrentPassword(e.target.value)}
                  rightElement={
                    <button
                      type="button"
                      onClick={() => setShowPassword(!showPassword)}
                      className="p-1 hover:opacity-70 transition"
                      tabIndex={-1}
                    >
                      {showPassword ? (
                        <EyeOff className="w-4 h-4 text-gray-500" />
                      ) : (
                        <Eye className="w-4 h-4 text-gray-500" />
                      )}
                    </button>
                  }
                />
                <Input
                  type={showPassword ? 'text' : 'password'}
                  placeholder="New password"
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                />
                <Input
                  type={showPassword ? 'text' : 'password'}
                  placeholder="Confirm new password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') handleChangePassword()
                  }}
                />
                {error && <p className="text-sm text-red-400">{error}</p>}
                {success && <p className="text-sm text-green-400">Password changed successfully!</p>}
                <div className="flex gap-2">
                  <Button size="sm" onClick={handleChangePassword}>
                    Save
                  </Button>
                  <Button size="sm" variant="secondary" onClick={() => {
                    setShowChangePassword(false)
                    setError('')
                    setCurrentPassword('')
                    setNewPassword('')
                    setConfirmPassword('')
                  }}>
                    Cancel
                  </Button>
                </div>
              </div>
            )}
          </div>
        )}

        {/* Lock Now Button */}
        {hasPasswordSet && (
          <div className="pt-2 border-t border-vault-border">
            <button
              onClick={() => lock()}
              className="text-sm text-vault-text-secondary hover:text-vault-text transition"
            >
              Lock Now
            </button>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

export function Settings() {
  const [settings, setSettings] = useState<AppSettings>(loadSettings)
  const [saved, setSaved] = useState(false)
  const [editingRpc, setEditingRpc] = useState<number | null>(null)

  // Sync i18n store language on mount
  useEffect(() => {
    const savedLang = loadSettings().language as 'en' | 'zh' | 'ja'
    if (savedLang && savedLang !== useI18nStore.getState().language) {
      useI18nStore.getState().setLanguage(savedLang)
    }
  }, [])

  useEffect(() => {
    applyTheme(settings.theme)
  }, [settings.theme])

  useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)")
    const handler = () => { if (settings.theme === "system") applyTheme(settings.theme) }
    mq.addEventListener("change", handler)
    return () => mq.removeEventListener("change", handler)
  }, [settings.theme])

  const handleSave = useCallback(() => {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings))
    setSaved(true)
    setTimeout(() => setSaved(false), 2000)
  }, [settings])

  const handleReset = useCallback(() => {
    setSettings(defaultSettings)
    localStorage.removeItem(STORAGE_KEY)
    setSaved(false)
  }, [])

  const updateRpc = useCallback((chainId: number, updated: RpcConfig) => {
    setSettings(prev => ({
      ...prev,
      rpcConfigs: prev.rpcConfigs.map(rc => rc.chainId === chainId ? updated : rc),
    }))
  }, [])

  const handleRpcInputBlur = useCallback((chainId: number, value: string) => {
    setSettings(prev => ({
      ...prev,
      rpcConfigs: prev.rpcConfigs.map(rc =>
        rc.chainId === chainId ? { ...rc, customUrl: value } : rc
      ),
    }))
    setEditingRpc(null)
  }, [])

  return (
    <div className="p-4 sm:p-6 max-w-3xl mx-auto space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-vault-text">Settings</h1>
          <p className="text-sm text-gray-500 mt-0.5">Configure your wallet preferences</p>
        </div>
        <div className="flex items-center gap-2">
          <button
            onClick={handleReset}
            className="px-4 py-2 text-sm rounded-lg border border-vault-border text-vault-text-secondary hover:border-vault-error hover:text-vault-error transition"
          >
            Reset
          </button>
          <button
            onClick={handleSave}
            className="px-4 py-2 text-sm rounded-lg bg-vault-gradient text-white hover:opacity-90 transition"
          >
            {saved ? "Saved!" : "Save"}
          </button>
        </div>
      </div>

      {/* Language */}
      <Card>
        <CardHeader>
          <CardTitle>Language</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
            {I18N_LANGUAGES.map(lang => (
              <button
                key={lang.value}
                onClick={() => {
                  setSettings(prev => ({ ...prev, language: lang.value }))
                  useI18nStore.getState().setLanguage(lang.value)
                }}
                className={`px-3 py-2 text-sm rounded-lg border transition ${
                  settings.language === lang.value
                    ? "border-vault-gradient bg-vault-gradient/10 text-vault-text"
                    : "border-vault-border text-vault-text-secondary hover:border-vault-gradient/50"
                }`}
              >
                {lang.label}
              </button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Currency */}
      <Card>
        <CardHeader>
          <CardTitle>Display Currency</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 sm:grid-cols-5 gap-2">
            {CURRENCIES.map(cur => (
              <button
                key={cur.value}
                onClick={() => setSettings(prev => ({ ...prev, currency: cur.value }))}
                className={`px-3 py-2 text-sm rounded-lg border transition ${
                  settings.currency === cur.value
                    ? "border-vault-gradient bg-vault-gradient/10 text-vault-text"
                    : "border-vault-border text-vault-text-secondary hover:border-vault-gradient/50"
                }`}
              >
                {cur.symbol} {cur.label}
              </button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Theme */}
      <Card>
        <CardHeader>
          <CardTitle>Theme</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-3 gap-2">
            {THEMES.map(t => (
              <button
                key={t.value}
                onClick={() => setSettings(prev => ({ ...prev, theme: t.value }))}
                className={`px-3 py-2 text-sm rounded-lg border transition ${
                  settings.theme === t.value
                    ? "border-vault-gradient bg-vault-gradient/10 text-vault-text"
                    : "border-vault-border text-vault-text-secondary hover:border-vault-gradient/50"
                }`}
              >
                {t.label}
              </button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* RPC Management */}
      <Card>
        <CardHeader>
          <CardTitle>Custom RPC URLs</CardTitle>
          <span className="text-xs text-vault-text-secondary">Per-chain override</span>
        </CardHeader>
        <CardContent>
          {settings.rpcConfigs.map(rc => {
            const chain = CHAINS.find(c => c.id === rc.chainId)
            if (!chain) return null
            return (
              <div key={rc.chainId}>
                {editingRpc === rc.chainId ? (
                  <div className="py-3 border-b border-vault-border last:border-0 space-y-2">
                    <div className="flex items-center gap-2">
                      <span className="text-sm font-medium text-vault-text">{chain.name}</span>
                    </div>
                    <input
                      autoFocus
                      type="url"
                      defaultValue={rc.customUrl}
                      placeholder={chain.rpcUrl}
                      className="w-full bg-vault-bg border border-vault-border rounded-lg px-3 py-2 text-sm text-vault-text placeholder-gray-600 focus:outline-none focus:border-vault-gradient"
                      onBlur={e => handleRpcInputBlur(rc.chainId, e.target.value)}
                      onKeyDown={e => {
                        if (e.key === "Enter") (e.target as HTMLInputElement).blur()
                        if (e.key === "Escape") setEditingRpc(null)
                      }}
                    />
                  </div>
                ) : (
                  <ChainRpcRow
                    chainId={rc.chainId}
                    config={rc}
                    onChange={updated => updateRpc(rc.chainId, updated)}
                  />
                )}
                {!editingRpc && (
                  <button
                    onClick={() => setEditingRpc(rc.chainId)}
                    className="mt-1 mb-2 text-xs text-vault-text-secondary hover:text-vault-gradient transition ml-auto block"
                  >
                    {rc.customUrl ? "Edit RPC URL" : "Add custom RPC URL"}
                  </button>
                )}
              </div>
            )
          })}
        </CardContent>
      </Card>

      {/* App Lock Settings */}
      <AppLockSettings />

      {/* Raw JSON preview */}
      <details className="group">
        <summary className="cursor-pointer text-xs text-vault-text-secondary hover:text-vault-text list-none flex items-center gap-1">
          <span className="transition group-open:rotate-90">▶</span>
          View raw settings JSON
        </summary>
        <pre className="mt-2 p-3 bg-vault-bg rounded-lg border border-vault-border text-xs text-vault-text-secondary overflow-auto max-h-48">
          {JSON.stringify(settings, null, 2)}
        </pre>
      </details>
    </div>
  )
}

export default Settings
