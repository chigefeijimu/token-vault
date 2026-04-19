export type BiometricType = 'fingerprint' | 'face' | 'iris' | 'none';

export interface SecuritySettings {
  isAppLockEnabled: boolean;
  isPinEnabled: boolean;
  isBiometricEnabled: boolean;
  biometricType: BiometricType;
  autoLockTimeout: number; // in milliseconds, 0 = immediate
  failedAttemptsLimit: number;
}

export interface AuthState {
  isAuthenticated: boolean;
  isLocked: boolean;
  lastActivity: number;
  sessionId?: string;
}

export interface LockScreenRequest {
  reason: 'manual' | 'timeout' | 'app_background' | 'failed_attempts';
}

export const DEFAULT_SECURITY_SETTINGS: SecuritySettings = {
  isAppLockEnabled: false,
  isPinEnabled: false,
  isBiometricEnabled: false,
  biometricType: 'none',
  autoLockTimeout: 60000, // 1 minute
  failedAttemptsLimit: 5,
};