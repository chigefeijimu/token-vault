import { create } from 'zustand';
import { persist } from 'zustand/middleware';

// Simple hash function using Web Crypto API for frontend-only storage
async function hashPassword(password: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(password);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  return hashArray.map(b => b.toString(16).padStart(2, '0')).join('');
}

async function verifyPassword(password: string, hash: string): Promise<boolean> {
  const passwordHash = await hashPassword(password);
  return passwordHash === hash;
}

interface SecurityState {
  isLocked: boolean;
  lockPasswordHash: string | null;
  lockTimeout: number; // in minutes
  lastActivity: number; // timestamp
  hasPasswordSet: boolean;

  setLockPassword: (password: string) => Promise<void>;
  verifyLockPassword: (password: string) => Promise<boolean>;
  lock: () => void;
  unlock: () => void;
  updateActivity: () => void;
  setLockTimeout: (minutes: number) => void;
  checkAutoLock: () => boolean;
}

export const useSecurityStore = create<SecurityState>()(
  persist(
    (set, get) => ({
      isLocked: false,
      lockPasswordHash: null,
      lockTimeout: 5, // default 5 minutes
      lastActivity: Date.now(),
      hasPasswordSet: false,

      setLockPassword: async (password: string) => {
        const hash = await hashPassword(password);
        set({
          lockPasswordHash: hash,
          hasPasswordSet: true,
          lastActivity: Date.now(),
        });
      },

      verifyLockPassword: async (password: string) => {
        const { lockPasswordHash } = get();
        if (!lockPasswordHash) return false;
        const isValid = await verifyPassword(password, lockPasswordHash);
        if (isValid) {
          set({ isLocked: false, lastActivity: Date.now() });
        }
        return isValid;
      },

      lock: () => {
        const { hasPasswordSet } = get();
        if (hasPasswordSet) {
          set({ isLocked: true });
        }
      },

      unlock: () => {
        set({ isLocked: false, lastActivity: Date.now() });
      },

      updateActivity: () => {
        set({ lastActivity: Date.now() });
      },

      setLockTimeout: (minutes: number) => {
        set({ lockTimeout: minutes });
      },

      checkAutoLock: () => {
        const { isLocked, hasPasswordSet, lockTimeout, lastActivity } = get();
        if (!hasPasswordSet || !isLocked) return false;
        
        const now = Date.now();
        const elapsed = now - lastActivity;
        const timeoutMs = lockTimeout * 60 * 1000;
        
        return elapsed >= timeoutMs;
      },
    }),
    {
      name: 'token-vault-security',
      partialize: (state) => ({
        lockPasswordHash: state.lockPasswordHash,
        lockTimeout: state.lockTimeout,
        hasPasswordSet: state.hasPasswordSet,
      }),
    }
  )
);
