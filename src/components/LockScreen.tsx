import { useState, useCallback } from 'react';
import { useSecurityStore } from '../stores/securityStore';
import { Button } from './common/Button';
import { Input } from './common/Input';
import { Card, CardContent, CardHeader, CardTitle } from './common/Card';
import { Lock, Shield, Eye, EyeOff } from 'lucide-react';

export function LockScreen() {
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [error, setError] = useState('');
  const [isLoading, setIsLoading] = useState(false);
  const [showPassword, setShowPassword] = useState(false);

  const isLocked = useSecurityStore(state => state.isLocked);
  const hasPasswordSet = useSecurityStore(state => state.hasPasswordSet);
  const verifyLockPassword = useSecurityStore(state => state.verifyLockPassword);
  const setLockPassword = useSecurityStore(state => state.setLockPassword);
  const lock = useSecurityStore(state => state.lock);

  const handleUnlock = useCallback(async () => {
    if (!password) {
      setError('Please enter your password');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      const isValid = await verifyLockPassword(password);
      if (!isValid) {
        setError('Incorrect password');
        setPassword('');
      }
    } catch {
      setError('Failed to verify password');
    } finally {
      setIsLoading(false);
    }
  }, [password, verifyLockPassword]);

  const handleSetPassword = useCallback(async () => {
    if (!password) {
      setError('Please enter a password');
      return;
    }
    if (password.length < 4) {
      setError('Password must be at least 4 characters');
      return;
    }
    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    setIsLoading(true);
    setError('');

    try {
      await setLockPassword(password);
      setPassword('');
      setConfirmPassword('');
    } catch {
      setError('Failed to set password');
    } finally {
      setIsLoading(false);
    }
  }, [password, confirmPassword, setLockPassword]);

  // Don't render if no password is set yet and we're not locked
  if (!hasPasswordSet && !isLocked) {
    return null;
  }

  const isSettingPassword = !hasPasswordSet;

  return (
    <div className="fixed inset-0 z-[9999] flex items-center justify-center bg-vault-bg/95 backdrop-blur-sm">
      {/* Background gradient */}
      <div className="absolute inset-0 bg-gradient-radial from-vault-gradient/10 to-transparent" />
      
      {/* Decorative elements */}
      <div className="absolute top-1/4 left-1/4 w-64 h-64 bg-vault-gradient/5 rounded-full blur-3xl" />
      <div className="absolute bottom-1/4 right-1/4 w-64 h-64 bg-vault-gradient/5 rounded-full blur-3xl" />

      <Card className="relative w-full max-w-md mx-4 shadow-2xl border-vault-border/50">
        <CardHeader className="text-center pb-2">
          <div className="flex justify-center mb-4">
            <div className="p-4 rounded-full bg-vault-gradient/20">
              {isSettingPassword ? (
                <Shield className="w-10 h-10 text-vault-gradient" />
              ) : (
                <Lock className="w-10 h-10 text-vault-gradient" />
              )}
            </div>
          </div>
          <CardTitle className="text-2xl">
            {isSettingPassword ? 'Set App Lock' : 'Vault Locked'}
          </CardTitle>
          <p className="text-sm text-gray-500 mt-1">
            {isSettingPassword 
              ? 'Create a password to secure your wallet'
              : 'Enter your password to access TokenVault'
            }
          </p>
        </CardHeader>
        
        <CardContent className="space-y-4 pt-4">
          {isSettingPassword ? (
            <>
              <div className="space-y-4">
                <div className="relative">
                  <Input
                    type={showPassword ? 'text' : 'password'}
                    placeholder="Enter password"
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && password && confirmPassword) {
                        handleSetPassword();
                      }
                    }}
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
                </div>
                <Input
                  type={showPassword ? 'text' : 'password'}
                  placeholder="Confirm password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && password && confirmPassword) {
                      handleSetPassword();
                    }
                  }}
                  hint="Minimum 4 characters"
                />
              </div>
              
              {error && (
                <p className="text-sm text-red-400 text-center">{error}</p>
              )}

              <Button
                onClick={handleSetPassword}
                isLoading={isLoading}
                disabled={!password || !confirmPassword}
                className="w-full"
                size="lg"
              >
                Set Password
              </Button>

              <p className="text-xs text-gray-500 text-center">
                This password will be required to access your wallet
              </p>
            </>
          ) : (
            <>
              <Input
                type={showPassword ? 'text' : 'password'}
                placeholder="Enter password"
                value={password}
                onChange={(e) => {
                  setPassword(e.target.value);
                  setError('');
                }}
                onKeyDown={(e) => {
                  if (e.key === 'Enter') {
                    handleUnlock();
                  }
                }}
                error={error}
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

              <Button
                onClick={handleUnlock}
                isLoading={isLoading}
                disabled={!password}
                className="w-full"
                size="lg"
              >
                Unlock
              </Button>

              <button
                onClick={() => lock()}
                className="w-full text-xs text-gray-500 hover:text-gray-400 transition mt-2"
              >
                Lock anyway (dev mode)
              </button>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

export default LockScreen;
