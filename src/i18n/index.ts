import { create } from 'zustand'
import { persist } from 'zustand/middleware'
import en from './locales/en.json'
import zh from './locales/zh.json'
import ja from './locales/ja.json'

export type Language = 'en' | 'zh' | 'ja'

const translations: Record<Language, typeof en> = { en, zh, ja }

export const LANGUAGES: { value: Language; label: string }[] = [
  { value: 'en', label: 'English' },
  { value: 'zh', label: '简体中文' },
  { value: 'ja', label: '日本語' },
]

interface I18nStore {
  language: Language
  setLanguage: (lang: Language) => void
  t: (key: string) => string
}

export const useI18nStore = create<I18nStore>()(
  persist(
    (set, get) => ({
      language: 'en',
      setLanguage: (language) => set({ language }),
      t: (key: string) => {
        const { language } = get()
        const keys = key.split('.')
        let value: unknown = translations[language]
        for (const k of keys) {
          if (value && typeof value === 'object' && k in value) {
            value = (value as Record<string, unknown>)[k]
          } else {
            // Fallback to English
            value = translations.en
            for (const k2 of keys) {
              if (value && typeof value === 'object' && k2 in value) {
                value = (value as Record<string, unknown>)[k2]
              } else {
                return key // Return key if not found
              }
            }
            break
          }
        }
        return typeof value === 'string' ? value : key
      },
    }),
    {
      name: 'token-vault-i18n',
    }
  )
)
