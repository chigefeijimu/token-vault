// Security module for app lock, PIN code, and biometric authentication

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    #[error("PIN not set up")]
    PinNotSet,
    #[error("PIN is locked due to too many failed attempts")]
    PinLocked,
    #[error("Invalid PIN")]
    InvalidPin,
    #[error("Biometric not available: {0}")]
    BiometricNotAvailable(String),
    #[error("Biometric authentication failed: {0}")]
    BiometricFailed(String),
    #[error("Security settings error: {0}")]
    SettingsError(String),
    #[error("App is locked")]
    AppLocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BiometricType {
    Fingerprint,
    Face,
    Iris,
    None,
}

impl Default for BiometricType {
    fn default() -> Self {
        BiometricType::None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub is_app_lock_enabled: bool,
    pub is_pin_enabled: bool,
    pub is_biometric_enabled: bool,
    pub biometric_type: BiometricType,
    pub auto_lock_timeout: u64, // in milliseconds, 0 = immediate
    pub failed_attempts_limit: u32,
    pub pin_hash: Option<String>,
    pub pin_salt: Option<String>,
}

impl Default for SecuritySettings {
    fn default() -> Self {
        SecuritySettings {
            is_app_lock_enabled: false,
            is_pin_enabled: false,
            is_biometric_enabled: false,
            biometric_type: BiometricType::None,
            auto_lock_timeout: 60000,
            failed_attempts_limit: 5,
            pin_hash: None,
            pin_salt: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub is_locked: bool,
    pub last_activity: u64,
    pub session_id: Option<String>,
    pub failed_attempts: u32,
}

impl Default for AuthState {
    fn default() -> Self {
        AuthState {
            is_authenticated: false,
            is_locked: false,
            last_activity: 0,
            session_id: None,
            failed_attempts: 0,
        }
    }
}

pub struct SecurityManager {
    settings: Mutex<SecuritySettings>,
    auth_state: Mutex<AuthState>,
    lock_time: Mutex<Option<Instant>>,
}

impl SecurityManager {
    pub fn new() -> Self {
        SecurityManager {
            settings: Mutex::new(SecuritySettings::default()),
            auth_state: Mutex::new(AuthState::default()),
            lock_time: Mutex::new(None),
        }
    }

    pub fn set_settings(&self, settings: SecuritySettings) {
        let mut current = self.settings.lock().unwrap();
        *current = settings;
    }

    pub fn get_settings(&self) -> SecuritySettings {
        self.settings.lock().unwrap().clone()
    }

    pub fn get_auth_state(&self) -> AuthState {
        self.auth_state.lock().unwrap().clone()
    }

    pub fn is_locked(&self) -> bool {
        self.auth_state.lock().unwrap().is_locked
    }

    pub fn is_authenticated(&self) -> bool {
        self.auth_state.lock().unwrap().is_authenticated
    }

    fn hash_pin(pin: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}${}", pin, salt));
        let result = hasher.finalize();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result)
    }

    fn generate_salt() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mut hasher = Sha256::new();
        hasher.update(timestamp.to_le_bytes());
        let result = hasher.finalize();
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result)
    }

    pub fn setup_pin(&self, pin: String) -> Result<(), SecurityError> {
        if pin.len() < 4 || pin.len() > 8 {
            return Err(SecurityError::AuthenticationFailed(
                "PIN must be 4-8 digits".to_string(),
            ));
        }

        let salt = Self::generate_salt();
        let hash = Self::hash_pin(&pin, &salt);

        let mut settings = self.settings.lock().unwrap();
        settings.pin_hash = Some(hash);
        settings.pin_salt = Some(salt);
        settings.is_pin_enabled = true;

        Ok(())
    }

    pub fn verify_pin(&self, pin: String) -> Result<bool, SecurityError> {
        let settings = self.settings.lock().unwrap();

        if !settings.is_pin_enabled {
            return Err(SecurityError::PinNotSet);
        }

        let hash = settings.pin_hash.clone().ok_or(SecurityError::PinNotSet)?;
        let salt = settings.pin_salt.clone().ok_or(SecurityError::PinNotSet)?;

        let mut auth_state = self.auth_state.lock().unwrap();
        
        // Check if locked due to failed attempts
        if auth_state.failed_attempts >= settings.failed_attempts_limit {
            return Err(SecurityError::PinLocked);
        }

        let input_hash = Self::hash_pin(&pin, &salt);
        
        if input_hash == hash {
            auth_state.is_authenticated = true;
            auth_state.is_locked = false;
            auth_state.failed_attempts = 0;
            auth_state.session_id = Some(uuid::Uuid::new_v4().to_string());
            auth_state.last_activity = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            Ok(true)
        } else {
            auth_state.failed_attempts += 1;
            Ok(false)
        }
    }

    pub fn verify_biometric(&self) -> Result<bool, SecurityError> {
        let settings = self.settings.lock().unwrap();

        if !settings.is_biometric_enabled {
            return Err(SecurityError::BiometricNotAvailable(
                "Biometric authentication is not enabled".to_string(),
            ));
        }

        // In a real implementation, this would use platform-specific biometric APIs
        // For now, we simulate successful biometric auth
        let mut auth_state = self.auth_state.lock().unwrap();
        auth_state.is_authenticated = true;
        auth_state.is_locked = false;
        auth_state.failed_attempts = 0;
        auth_state.session_id = Some(uuid::Uuid::new_v4().to_string());
        auth_state.last_activity = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(true)
    }

    pub fn lock_app(&self) {
        let mut auth_state = self.auth_state.lock().unwrap();
        auth_state.is_authenticated = false;
        auth_state.is_locked = true;
        auth_state.session_id = None;
        
        let mut lock_time = self.lock_time.lock().unwrap();
        *lock_time = Some(Instant::now());
    }

    pub fn unlock_app(&self) -> Result<(), SecurityError> {
        let settings = self.settings.lock().unwrap();
        
        if !settings.is_app_lock_enabled {
            return Ok(());
        }

        let mut auth_state = self.auth_state.lock().unwrap();
        auth_state.is_locked = false;
        
        let mut lock_time = self.lock_time.lock().unwrap();
        *lock_time = None;
        
        Ok(())
    }

    pub fn update_activity(&self) {
        let mut auth_state = self.auth_state.lock().unwrap();
        auth_state.last_activity = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    pub fn check_auto_lock(&self) -> bool {
        let settings = self.settings.lock().unwrap();
        let auth_state = self.auth_state.lock().unwrap();

        if !settings.is_app_lock_enabled || auth_state.is_locked {
            return false;
        }

        if settings.auto_lock_timeout == 0 {
            return true;
        }

        let last_activity = auth_state.last_activity;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now - last_activity > settings.auto_lock_timeout / 1000 {
            drop(auth_state);
            self.lock_app();
            return true;
        }

        false
    }

    pub fn disable_pin(&self) {
        let mut settings = self.settings.lock().unwrap();
        settings.is_pin_enabled = false;
        settings.pin_hash = None;
        settings.pin_salt = None;
    }

    pub fn enable_biometric(&self) -> Result<(), SecurityError> {
        let mut settings = self.settings.lock().unwrap();
        settings.is_biometric_enabled = true;
        // In production, detect actual biometric type
        settings.biometric_type = BiometricType::Fingerprint;
        Ok(())
    }

    pub fn disable_biometric(&self) {
        let mut settings = self.settings.lock().unwrap();
        settings.is_biometric_enabled = false;
        settings.biometric_type = BiometricType::None;
    }

    pub fn enable_app_lock(&self) {
        let mut settings = self.settings.lock().unwrap();
        settings.is_app_lock_enabled = true;
    }

    pub fn disable_app_lock(&self) {
        let mut settings = self.settings.lock().unwrap();
        settings.is_app_lock_enabled = false;
    }

    pub fn reset_failed_attempts(&self) {
        let mut auth_state = self.auth_state.lock().unwrap();
        auth_state.failed_attempts = 0;
    }

    pub fn get_remaining_attempts(&self) -> u32 {
        let settings = self.settings.lock().unwrap();
        let auth_state = self.auth_state.lock().unwrap();
        settings.failed_attempts_limit.saturating_sub(auth_state.failed_attempts)
    }
}

// Tauri commands

#[tauri::command]
pub fn setup_pin_code(pin: String, state: tauri::State<'_, SecurityManager>) -> Result<(), String> {
    state
        .setup_pin(pin)
        .map(|_| ())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn verify_pin_code(pin: String, state: tauri::State<'_, SecurityManager>) -> Result<bool, String> {
    state
        .verify_pin(pin)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn verify_biometric(state: tauri::State<'_, SecurityManager>) -> Result<bool, String> {
    state
        .verify_biometric()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn lock_app(state: tauri::State<'_, SecurityManager>) -> Result<(), String> {
    state.lock_app();
    Ok(())
}

#[tauri::command]
pub fn unlock_app(state: tauri::State<'_, SecurityManager>) -> Result<(), String> {
    state
        .unlock_app()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_auth_state(state: tauri::State<'_, SecurityManager>) -> AuthState {
    state.get_auth_state()
}

#[tauri::command]
pub fn get_security_settings(state: tauri::State<'_, SecurityManager>) -> SecuritySettings {
    state.get_settings()
}

#[tauri::command]
pub fn update_security_settings(
    settings: SecuritySettings,
    state: tauri::State<'_, SecurityManager>,
) -> Result<(), String> {
    state.set_settings(settings);
    Ok(())
}

#[tauri::command]
pub fn enable_app_lock(state: tauri::State<'_, SecurityManager>) -> Result<(), String> {
    state.enable_app_lock();
    Ok(())
}

#[tauri::command]
pub fn disable_app_lock(state: tauri::State<'_, SecurityManager>) -> Result<(), String> {
    state.disable_app_lock();
    Ok(())
}

#[tauri::command]
pub fn enable_biometric(state: tauri::State<'_, SecurityManager>) -> Result<(), String> {
    state
        .enable_biometric()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn disable_biometric(state: tauri::State<'_, SecurityManager>) -> Result<(), String> {
    state.disable_biometric();
    Ok(())
}

#[tauri::command]
pub fn disable_pin_code(state: tauri::State<'_, SecurityManager>) -> Result<(), String> {
    state.disable_pin();
    Ok(())
}

#[tauri::command]
pub fn update_activity(state: tauri::State<'_, SecurityManager>) -> Result<(), String> {
    state.update_activity();
    Ok(())
}

#[tauri::command]
pub fn check_auto_lock(state: tauri::State<'_, SecurityManager>) -> Result<bool, String> {
    Ok(state.check_auto_lock())
}

#[tauri::command]
pub fn get_remaining_attempts(state: tauri::State<'_, SecurityManager>) -> u32 {
    state.get_remaining_attempts()
}

#[tauri::command]
pub fn reset_failed_attempts(state: tauri::State<'_, SecurityManager>) -> Result<(), String> {
    state.reset_failed_attempts();
    Ok(())
}