import { useCallback, useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { AuthState, SecuritySettings, DEFAULT_SECURITY_SETTINGS } from '../types/security';

const DEFAULT_SETTINGS: SecuritySettings = {
  isAppLockEnabled: false,
  isPinEnabled: false,
  isBiometricEnabled: false,
  biometricType: 'none',
  autoLockTimeout: 60000,
  failedAttemptsLimit: 5,
};

export interface UseSecurityReturn {
  authState: AuthState | null;
  securitySettings: SecuritySettings;
  isLocked: boolean;
  isAuthenticated: boolean;
  remainingAttempts: number;
  setupPin: (pin: string) => Promise<boolean>;
  verifyPin: (pin: string) => Promise<boolean>;
  verifyBiometric: () => Promise<boolean>;
  lockApp: () => Promise<void>;
  unlockApp: () => Promise<void>;
  enableAppLock: () => Promise<void>;
  disableAppLock: () => Promise<void>;
  enableBiometric: () => Promise<void>;
  disableBiometric: () => Promise<void>;
  disablePin: () => Promise<void>;
  updateActivity: () => Promise<void>;
  checkAutoLock: () => Promise<boolean>;
  resetFailedAttempts: () => Promise<void>;
  refreshAuthState: () => Promise<void>;
  refreshSettings: () => Promise<void>;
}

export function useSecurity(): UseSecurityReturn {
  const [authState, setAuthState] = useState<AuthState | null>(null);
  const [securitySettings, setSecuritySettings] = useState<SecuritySettings>(DEFAULT_SETTINGS);
  const [remainingAttempts, setRemainingAttempts] = useState<number>(5);

  const refreshAuthState = useCallback(async () => {
    try {
      const state = await invoke<AuthState>('get_auth_state');
      setAuthState(state);
      const remaining = await invoke<number>('get_remaining_attempts');
      setRemainingAttempts(remaining);
    } catch (error) {
      console.error('Failed to refresh auth state:', error);
    }
  }, []);

  const refreshSettings = useCallback(async () => {
    try {
      const settings = await invoke<SecuritySettings>('get_security_settings');
      setSecuritySettings(settings);
    } catch (error) {
      console.error('Failed to refresh settings:', error);
    }
  }, []);

  useEffect(() => {
    refreshAuthState();
    refreshSettings();

    let unlistenAutoLock: UnlistenFn | undefined;

    const setupAutoLockListener = async () => {
      unlistenAutoLock = await listen<boolean>('auto-lock-triggered', (event) => {
        if (event.payload) {
          refreshAuthState();
        }
      });
    };

    setupAutoLockListener();

    return () => {
      if (unlistenAutoLock) {
        unlistenAutoLock();
      }
    };
  }, [refreshAuthState, refreshSettings]);

  // Auto-check for lock timeout
  useEffect(() => {
    const interval = setInterval(async () => {
      try {
        const shouldLock = await invoke<boolean>('check_auto_lock');
        if (shouldLock) {
          refreshAuthState();
        }
      } catch (error) {
        console.error('Auto-lock check failed:', error);
      }
    }, 10000); // Check every 10 seconds

    return () => clearInterval(interval);
  }, [refreshAuthState]);

  const setupPin = useCallback(async (pin: string): Promise<boolean> => {
    try {
      const result = await invoke<boolean>('setup_pin_code', { pin });
      await refreshSettings();
      return result;
    } catch (error) {
      console.error('Failed to setup PIN:', error);
      throw error;
    }
  }, [refreshSettings]);

  const verifyPin = useCallback(async (pin: string): Promise<boolean> => {
    try {
      const result = await invoke<boolean>('verify_pin_code', { pin });
      await refreshAuthState();
      return result;
    } catch (error) {
      console.error('Failed to verify PIN:', error);
      throw error;
    }
  }, [refreshAuthState]);

  const verifyBiometric = useCallback(async (): Promise<boolean> => {
    try {
      const result = await invoke<boolean>('verify_biometric');
      await refreshAuthState();
      return result;
    } catch (error) {
      console.error('Failed to verify biometric:', error);
      throw error;
    }
  }, [refreshAuthState]);

  const lockApp = useCallback(async (): Promise<void> => {
    try {
      await invoke('lock_app');
      await refreshAuthState();
    } catch (error) {
      console.error('Failed to lock app:', error);
      throw error;
    }
  }, [refreshAuthState]);

  const unlockApp = useCallback(async (): Promise<void> => {
    try {
      await invoke('unlock_app');
      await refreshAuthState();
    } catch (error) {
      console.error('Failed to unlock app:', error);
      throw error;
    }
  }, [refreshAuthState]);

  const enableAppLock = useCallback(async (): Promise<void> => {
    try {
      await invoke('enable_app_lock');
      await refreshSettings();
    } catch (error) {
      console.error('Failed to enable app lock:', error);
      throw error;
    }
  }, [refreshSettings]);

  const disableAppLock = useCallback(async (): Promise<void> => {
    try {
      await invoke('disable_app_lock');
      await refreshSettings();
    } catch (error) {
      console.error('Failed to disable app lock:', error);
      throw error;
    }
  }, [refreshSettings]);

  const enableBiometric = useCallback(async (): Promise<void> => {
    try {
      await invoke('enable_biometric');
      await refreshSettings();
    } catch (error) {
      console.error('Failed to enable biometric:', error);
      throw error;
    }
  }, [refreshSettings]);

  const disableBiometric = useCallback(async (): Promise<void> => {
    try {
      await invoke('disable_biometric');
      await refreshSettings();
    } catch (error) {
      console.error('Failed to disable biometric:', error);
      throw error;
    }
  }, [refreshSettings]);

  const disablePin = useCallback(async (): Promise<void> => {
    try {
      await invoke('disable_pin_code');
      await refreshSettings();
    } catch (error) {
      console.error('Failed to disable PIN:', error);
      throw error;
    }
  }, [refreshSettings]);

  const updateActivity = useCallback(async (): Promise<void> => {
    try {
      await invoke('update_activity');
    } catch (error) {
      console.error('Failed to update activity:', error);
    }
  }, []);

  const checkAutoLock = useCallback(async (): Promise<boolean> => {
    try {
      return await invoke<boolean>('check_auto_lock');
    } catch (error) {
      console.error('Failed to check auto-lock:', error);
      return false;
    }
  }, []);

  const resetFailedAttempts = useCallback(async (): Promise<void> => {
    try {
      await invoke('reset_failed_attempts');
      await refreshAuthState();
    } catch (error) {
      console.error('Failed to reset failed attempts:', error);
      throw error;
    }
  }, [refreshAuthState]);

  return {
    authState,
    securitySettings,
    isLocked: authState?.isLocked ?? false,
    isAuthenticated: authState?.isAuthenticated ?? false,
    remainingAttempts,
    setupPin,
    verifyPin,
    verifyBiometric,
    lockApp,
    unlockApp,
    enableAppLock,
    disableAppLock,
    enableBiometric,
    disableBiometric,
    disablePin,
    updateActivity,
    checkAutoLock,
    resetFailedAttempts,
    refreshAuthState,
    refreshSettings,
  };
}