import { useState, useEffect, useCallback } from "react"
import { Card, CardHeader, CardTitle, CardContent } from "../components/common/Card"
import { CHAINS } from "../types/wallet"

const DEFAULT_CHAIN_IDS = [1, 56, 137, 42161, 10, 43114]

const LANGUAGES = [
  { value: "en", label: "English" },
  { value: "zh", label: "简体中文" },
  { value: "ja", label: "日本語" },
  { value: "es", label: "Español" },
]

const CURRENCIES = [
  { value: "USD", label: "USD", symbol: "$" },
  { value: "CNY", label: "CNY", symbol: "¥" },
  { value: "JPY", label: "JPY", symbol: "¥" },
  { value: "EUR", label: "EUR", symbol: "€" },
  { value: "GBP", label: "GBP", symbol: "£" },
]

const THEMES = [
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
  { value: "system", label: "System" },
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

export function Settings() {
  const [settings, setSettings] = useState<AppSettings>(loadSettings)
  const [saved, setSaved] = useState(false)
  const [editingRpc, setEditingRpc] = useState<number | null>(null)

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
            {LANGUAGES.map(lang => (
              <button
                key={lang.value}
                onClick={() => setSettings(prev => ({ ...prev, language: lang.value }))}
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
