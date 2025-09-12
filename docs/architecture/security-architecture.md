# Security Architecture

## Overview

BackTestr_ai implements a defense-in-depth security approach tailored for a local-only desktop application handling sensitive financial data. The security architecture focuses on protecting user data, ensuring algorithm execution integrity, and maintaining system isolation without relying on external authentication services.

## 1. Application Security

### Code Signing & Integrity

```rust
// Application integrity verification
pub struct ApplicationSecurity {
    signature_validator: SignatureValidator,
    checksum_verifier: ChecksumVerifier,
    tamper_detection: TamperDetection,
}

impl ApplicationSecurity {
    pub fn verify_application_integrity(&self) -> SecurityResult<()> {
        // Verify code signature
        self.signature_validator.verify_signature()?;
        
        // Check file integrity
        self.checksum_verifier.verify_checksums()?;
        
        // Detect tampering
        self.tamper_detection.check_modifications()?;
        
        Ok(())
    }
}
```

**Key Security Measures:**
- **Code Signing**: All executables signed with company certificate
- **Integrity Checks**: Runtime verification of critical binaries
- **Tamper Detection**: Monitor for unauthorized file modifications
- **Secure Updates**: Cryptographically signed update packages

### Binary Protection

```rust
// Binary hardening configuration
#[cfg(windows)]
const SECURITY_FLAGS: &[&str] = &[
    "/DYNAMICBASE",     // ASLR
    "/NXCOMPAT",        // DEP
    "/GUARD:CF",        // Control Flow Guard
    "/HIGHENTROPYVA",   // High entropy ASLR
];

// Runtime security checks
pub fn enable_security_features() {
    #[cfg(windows)]
    unsafe {
        // Enable DEP for current process
        let mut flags = 0u32;
        SetProcessDEPPolicy(PROCESS_DEP_ENABLE | PROCESS_DEP_DISABLE_ATL_THUNK_EMULATION);
    }
}
```

## 2. Data Security

### Encryption at Rest

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub struct DataEncryption {
    cipher: Aes256Gcm,
    key_derivation: Argon2<'static>,
}

impl DataEncryption {
    pub fn encrypt_sensitive_data(&self, data: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
        // Derive key from password using Argon2
        let salt = self.generate_salt();
        let key = self.derive_key(password, &salt)?;
        
        // Encrypt with AES-256-GCM
        let cipher = Aes256Gcm::new(&key);
        let nonce = self.generate_nonce();
        let ciphertext = cipher.encrypt(&nonce, data)?;
        
        // Combine salt, nonce, and ciphertext
        Ok(self.combine_encrypted_parts(salt, nonce, ciphertext))
    }
    
    pub fn decrypt_sensitive_data(&self, encrypted_data: &[u8], password: &str) -> Result<Vec<u8>, CryptoError> {
        let (salt, nonce, ciphertext) = self.split_encrypted_parts(encrypted_data)?;
        let key = self.derive_key(password, &salt)?;
        
        let cipher = Aes256Gcm::new(&key);
        cipher.decrypt(&nonce, ciphertext).map_err(Into::into)
    }
}
```

**Data Protection Strategy:**
- **Database Encryption**: SQLCipher for DuckDB encryption
- **Memory Protection**: Secure memory allocation for sensitive data
- **Key Management**: PBKDF2/Argon2 for key derivation
- **Secure Deletion**: Memory wiping after use

### Secure Storage Implementation

```rust
// Secure local storage
pub struct SecureStorage {
    db_path: PathBuf,
    encryption_key: Option<[u8; 32]>,
    access_control: FilePermissions,
}

impl SecureStorage {
    pub fn initialize_secure_database(&mut self, master_password: &str) -> Result<(), StorageError> {
        // Set restrictive file permissions (owner read/write only)
        self.set_database_permissions()?;
        
        // Initialize encrypted database
        let connection_string = format!(
            "{}?key='{}'&cipher_page_size=4096&cipher_hmac_algorithm=HMAC_SHA512",
            self.db_path.display(),
            self.derive_database_key(master_password)?
        );
        
        self.connect_encrypted_database(&connection_string)
    }
    
    fn set_database_permissions(&self) -> Result<(), std::io::Error> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = std::fs::Permissions::from_mode(0o600); // Owner read/write only
            std::fs::set_permissions(&self.db_path, perms)?;
        }
        
        #[cfg(windows)]
        {
            // Set ACL for Windows
            self.set_windows_acl()?;
        }
        
        Ok(())
    }
}
```

## 3. Algorithm Sandboxing

### Python Execution Isolation

```rust
use pyo3::prelude::*;
use std::collections::HashMap;

pub struct PythonSandbox {
    restricted_modules: HashSet<String>,
    allowed_builtins: HashSet<String>,
    resource_limits: ResourceLimits,
    execution_timeout: Duration,
}

impl PythonSandbox {
    pub fn new() -> Self {
        let mut restricted_modules = HashSet::new();
        restricted_modules.extend([
            "subprocess".to_string(),
            "os".to_string(),
            "sys".to_string(),
            "importlib".to_string(),
            "socket".to_string(),
            "urllib".to_string(),
            "requests".to_string(),
            "http".to_string(),
            "__import__".to_string(),
        ]);
        
        let mut allowed_builtins = HashSet::new();
        allowed_builtins.extend([
            "len".to_string(),
            "range".to_string(),
            "abs".to_string(),
            "min".to_string(),
            "max".to_string(),
            "sum".to_string(),
            "round".to_string(),
        ]);
        
        Self {
            restricted_modules,
            allowed_builtins,
            resource_limits: ResourceLimits::default(),
            execution_timeout: Duration::from_secs(30),
        }
    }
    
    pub fn execute_algorithm(&self, code: &str, context: AlgorithmContext) -> Result<AlgorithmResult, ExecutionError> {
        Python::with_gil(|py| {
            // Create restricted environment
            let restricted_globals = self.create_restricted_globals(py)?;
            
            // Set up resource monitoring
            let resource_monitor = ResourceMonitor::new(self.resource_limits.clone());
            
            // Execute with timeout and monitoring
            let result = tokio::time::timeout(
                self.execution_timeout,
                self.execute_in_restricted_env(py, code, restricted_globals, context)
            ).await??;
            
            // Validate execution results
            self.validate_execution_result(&result)?;
            
            Ok(result)
        })
    }
    
    fn create_restricted_globals(&self, py: Python) -> PyResult<&PyDict> {
        let globals = PyDict::new(py);
        
        // Add only allowed built-ins
        let builtins = PyDict::new(py);
        for builtin in &self.allowed_builtins {
            if let Ok(func) = py.eval(&format!("__builtins__.{}", builtin), None, None) {
                builtins.set_item(builtin, func)?;
            }
        }
        globals.set_item("__builtins__", builtins)?;
        
        // Add safe mathematical libraries
        let numpy = py.import("numpy")?;
        let pandas = py.import("pandas")?;
        globals.set_item("np", numpy)?;
        globals.set_item("pd", pandas)?;
        
        Ok(globals)
    }
}

#[derive(Debug, Clone)]
pub struct ResourceLimits {
    max_memory_mb: usize,
    max_execution_time: Duration,
    max_file_descriptors: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_execution_time: Duration::from_secs(30),
            max_file_descriptors: 64,
        }
    }
}
```

**Sandboxing Features:**
- **Module Restrictions**: Block dangerous Python modules (os, subprocess, etc.)
- **Resource Limits**: Memory and CPU time constraints
- **Network Isolation**: No external network access
- **File System Restrictions**: Limited to data directories only

## 4. IPC Security

### Context Isolation

```typescript
// Electron preload script with secure context isolation
import { contextBridge, ipcRenderer } from 'electron';

// Security-hardened API exposure
const secureAPI = {
    // Data operations with validation
    data: {
        async query(query: string, params: unknown[]): Promise<QueryResult> {
            // Input validation
            if (!this.validateQuery(query)) {
                throw new Error('Invalid query detected');
            }
            
            return await ipcRenderer.invoke('secure-data-query', {
                query: this.sanitizeQuery(query),
                params: this.validateParams(params),
                requestId: this.generateRequestId(),
                timestamp: Date.now()
            });
        },
        
        validateQuery(query: string): boolean {
            // Prevent SQL injection attempts
            const dangerousPatterns = [
                /;\s*(drop|delete|update|insert)\s+/i,
                /union\s+select/i,
                /exec\s*\(/i,
                /script\s*:/i
            ];
            
            return !dangerousPatterns.some(pattern => pattern.test(query));
        }
    },
    
    // Algorithm execution with security checks
    algorithm: {
        async execute(code: string, config: AlgorithmConfig): Promise<AlgorithmResult> {
            // Code validation before execution
            if (!this.validateAlgorithmCode(code)) {
                throw new Error('Potentially dangerous code detected');
            }
            
            return await ipcRenderer.invoke('secure-algorithm-execute', {
                code: this.sanitizeCode(code),
                config: this.validateConfig(config),
                securityContext: this.createSecurityContext()
            });
        },
        
        validateAlgorithmCode(code: string): boolean {
            const forbiddenPatterns = [
                /import\s+(os|sys|subprocess|socket)/,
                /__import__\s*\(/,
                /exec\s*\(/,
                /eval\s*\(/,
                /open\s*\(/
            ];
            
            return !forbiddenPatterns.some(pattern => pattern.test(code));
        }
    }
};

// Expose API through context bridge
contextBridge.exposeInMainWorld('electronAPI', secureAPI);
```

**IPC Security Features:**
- **Context Isolation**: Complete separation between main and renderer processes
- **Input Validation**: All IPC messages validated and sanitized
- **Request Authentication**: Request signing and validation
- **Message Encryption**: Sensitive data encrypted in transit

### Secure Channel Implementation

```rust
// Secure IPC channel with encryption
pub struct SecureIPCChannel {
    channel_key: [u8; 32],
    sequence_number: AtomicU64,
    message_authenticator: MessageAuthenticator,
}

impl SecureIPCChannel {
    pub fn send_secure_message(&self, message: &IPCMessage) -> Result<(), IPCError> {
        // Serialize and encrypt message
        let serialized = self.serialize_message(message)?;
        let encrypted = self.encrypt_message(&serialized)?;
        
        // Add authentication tag
        let authenticated = self.message_authenticator.authenticate(&encrypted)?;
        
        // Send with sequence number for replay protection
        let secure_packet = SecurePacket {
            sequence: self.sequence_number.fetch_add(1, Ordering::SeqCst),
            payload: authenticated,
            timestamp: SystemTime::now(),
        };
        
        self.send_packet(&secure_packet)
    }
    
    pub fn receive_secure_message(&self) -> Result<IPCMessage, IPCError> {
        let packet = self.receive_packet()?;
        
        // Verify sequence number and timestamp
        self.verify_packet_freshness(&packet)?;
        
        // Verify authentication
        let decrypted = self.message_authenticator.verify_and_decrypt(&packet.payload)?;
        
        // Deserialize message
        self.deserialize_message(&decrypted)
    }
}
```

## 5. User Authentication

### Local Authentication System

```rust
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{rand_core::OsRng, SaltString}};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LocalAuthConfig {
    password_hash: String,
    salt: String,
    created_at: SystemTime,
    failed_attempts: u32,
    lockout_until: Option<SystemTime>,
}

pub struct LocalAuthenticator {
    config: LocalAuthConfig,
    max_attempts: u32,
    lockout_duration: Duration,
    session_timeout: Duration,
}

impl LocalAuthenticator {
    pub fn create_user(&mut self, password: &str) -> Result<(), AuthError> {
        // Validate password strength
        self.validate_password_strength(password)?;
        
        // Generate salt and hash password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| AuthError::HashingFailed)?
            .to_string();
        
        self.config = LocalAuthConfig {
            password_hash,
            salt: salt.to_string(),
            created_at: SystemTime::now(),
            failed_attempts: 0,
            lockout_until: None,
        };
        
        self.save_config()?;
        Ok(())
    }
    
    pub fn authenticate(&mut self, password: &str) -> Result<AuthSession, AuthError> {
        // Check if account is locked out
        if let Some(lockout_until) = self.config.lockout_until {
            if SystemTime::now() < lockout_until {
                return Err(AuthError::AccountLocked);
            } else {
                self.config.lockout_until = None;
                self.config.failed_attempts = 0;
            }
        }
        
        // Verify password
        let parsed_hash = PasswordHash::new(&self.config.password_hash)
            .map_err(|_| AuthError::InvalidHash)?;
        
        let argon2 = Argon2::default();
        match argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(_) => {
                // Reset failed attempts on successful login
                self.config.failed_attempts = 0;
                self.config.lockout_until = None;
                self.save_config()?;
                
                // Create authenticated session
                Ok(AuthSession {
                    session_id: self.generate_session_id(),
                    created_at: SystemTime::now(),
                    expires_at: SystemTime::now() + self.session_timeout,
                    user_authenticated: true,
                })
            }
            Err(_) => {
                // Increment failed attempts
                self.config.failed_attempts += 1;
                
                // Lock account if too many failures
                if self.config.failed_attempts >= self.max_attempts {
                    self.config.lockout_until = Some(SystemTime::now() + self.lockout_duration);
                }
                
                self.save_config()?;
                Err(AuthError::InvalidCredentials)
            }
        }
    }
    
    fn validate_password_strength(&self, password: &str) -> Result<(), AuthError> {
        if password.len() < 12 {
            return Err(AuthError::PasswordTooShort);
        }
        
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| "!@#$%^&*()_+-=[]{}|;:,.<>?".contains(c));
        
        if !has_uppercase || !has_lowercase || !has_digit || !has_special {
            return Err(AuthError::PasswordTooWeak);
        }
        
        Ok(())
    }
}

#[derive(Debug)]
pub struct AuthSession {
    pub session_id: String,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub user_authenticated: bool,
}
```

**Authentication Features:**
- **Local-Only**: No external authentication dependencies
- **Strong Password Policies**: Enforced complexity requirements
- **Account Lockout**: Protection against brute force attacks
- **Session Management**: Secure session tokens with expiration

## 6. Security Best Practices

### Secure Development Guidelines

```rust
// Security utilities and helpers
pub mod security_utils {
    use zeroize::Zeroize;
    
    /// Secure string that zeros memory on drop
    pub struct SecureString {
        data: Vec<u8>,
    }
    
    impl SecureString {
        pub fn new(data: String) -> Self {
            Self {
                data: data.into_bytes(),
            }
        }
        
        pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
            std::str::from_utf8(&self.data)
        }
    }
    
    impl Drop for SecureString {
        fn drop(&mut self) {
            self.data.zeroize();
        }
    }
    
    /// Input sanitization
    pub fn sanitize_input(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || " -_.,()[]{}".contains(*c))
            .collect()
    }
    
    /// Generate cryptographically secure random bytes
    pub fn generate_secure_random(size: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut rng = rand::rngs::OsRng;
        let mut bytes = vec![0u8; size];
        rng.fill_bytes(&mut bytes);
        bytes
    }
}
```

### Security Monitoring

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, warn, info};

#[derive(Debug, Serialize, Deserialize)]
pub enum SecurityEvent {
    AuthenticationFailure { timestamp: SystemTime, source: String },
    UnauthorizedAccess { timestamp: SystemTime, resource: String },
    SuspiciousActivity { timestamp: SystemTime, details: String },
    IntegrityViolation { timestamp: SystemTime, file_path: String },
    PrivilegeEscalation { timestamp: SystemTime, context: String },
}

pub struct SecurityMonitor {
    events: Vec<SecurityEvent>,
    alert_thresholds: HashMap<String, usize>,
    monitoring_enabled: bool,
}

impl SecurityMonitor {
    pub fn log_security_event(&mut self, event: SecurityEvent) {
        match &event {
            SecurityEvent::AuthenticationFailure { .. } => {
                warn!("Authentication failure detected: {:?}", event);
            }
            SecurityEvent::UnauthorizedAccess { .. } => {
                error!("Unauthorized access attempt: {:?}", event);
            }
            SecurityEvent::SuspiciousActivity { .. } => {
                warn!("Suspicious activity detected: {:?}", event);
            }
            SecurityEvent::IntegrityViolation { .. } => {
                error!("File integrity violation: {:?}", event);
            }
            SecurityEvent::PrivilegeEscalation { .. } => {
                error!("Privilege escalation attempt: {:?}", event);
            }
        }
        
        self.events.push(event);
        
        // Check if immediate action is required
        self.evaluate_threat_level();
    }
    
    fn evaluate_threat_level(&self) {
        let recent_events = self.events.iter()
            .filter(|e| self.is_recent_event(e))
            .count();
        
        if recent_events > 10 {
            error!("High security threat level detected - {} events in last hour", recent_events);
            // Could trigger additional security measures
        }
    }
}
```

**Security Best Practices:**
- **Principle of Least Privilege**: Minimal permissions for all components
- **Defense in Depth**: Multiple layers of security controls
- **Secure by Default**: Secure configurations out of the box
- **Regular Security Auditing**: Automated security monitoring and logging
- **Secure Memory Management**: Automatic cleanup of sensitive data
- **Input Validation**: Comprehensive sanitization of all user inputs

## Summary

The Security Architecture provides comprehensive protection for BackTestr_ai through:

1. **Application Integrity**: Code signing, tamper detection, and binary hardening
2. **Data Protection**: Strong encryption, secure storage, and proper key management
3. **Execution Isolation**: Sandboxed Python execution with resource limits
4. **Secure Communication**: Encrypted IPC with authentication and replay protection
5. **Local Authentication**: Strong password policies with account lockout protection
6. **Continuous Monitoring**: Security event logging and threat detection
