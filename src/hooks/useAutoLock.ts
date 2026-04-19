import { useEffect, useCallback, useRef } from 'react';
import { useSecurityStore } from '../stores/securityStore';

export function useAutoLock() {
  const isLocked = useSecurityStore(state => state.isLocked);
  const hasPasswordSet = useSecurityStore(state => state.hasPasswordSet);
  const lockTimeout = useSecurityStore(state => state.lockTimeout);
  const lastActivity = useSecurityStore(state => state.lastActivity);
  const lock = useSecurityStore(state => state.lock);
  const updateActivity = useSecurityStore(state => state.updateActivity);

  const lastActivityRef = useRef(lastActivity);
  lastActivityRef.current = lastActivity;

  const handleActivity = useCallback(() => {
    if (!isLocked && hasPasswordSet) {
      updateActivity();
    }
  }, [isLocked, hasPasswordSet, updateActivity]);

  // Set up activity listeners
  useEffect(() => {
    if (!hasPasswordSet) return;

    const events = ['mousemove', 'keydown', 'click', 'touchstart', 'scroll'];
    
    // Throttle activity updates
    let lastUpdate = Date.now();
    const throttledHandler = () => {
      const now = Date.now();
      if (now - lastUpdate > 5000) { // Update at most every 5 seconds
        lastUpdate = now;
        handleActivity();
      }
    };

    events.forEach(event => {
      document.addEventListener(event, throttledHandler, { passive: true });
    });

    return () => {
      events.forEach(event => {
        document.removeEventListener(event, throttledHandler);
      });
    };
  }, [hasPasswordSet, handleActivity]);

  // Check for auto-lock every 30 seconds
  useEffect(() => {
    if (!hasPasswordSet) return;

    const interval = setInterval(() => {
      const { isLocked: currentLocked, hasPasswordSet: currentHasPassword } = useSecurityStore.getState();
      if (!currentHasPassword) return;

      const now = Date.now();
      const elapsed = now - lastActivityRef.current;
      const timeoutMs = lockTimeout * 60 * 1000;

      // Lock if timeout exceeded and not already locked
      if (!currentLocked && elapsed >= timeoutMs) {
        lock();
      }
    }, 30000); // Check every 30 seconds

    return () => clearInterval(interval);
  }, [hasPasswordSet, lockTimeout, lock]);
}
