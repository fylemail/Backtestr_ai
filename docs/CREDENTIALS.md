# Credential Management

## Overview

BackTestr AI provides secure credential management for API keys, passwords, and other sensitive data. The system supports multiple storage backends with a focus on security best practices.

## Storage Backends

### Environment Variables (Development)
- Default for development environments
- Credentials stored as environment variables
- Easy to use but less secure
- Set via `.env.local` file (never commit this file)

### Windows Credential Manager (Production)
- Recommended for production use on Windows
- Credentials stored in Windows secure storage
- Encrypted at rest
- Accessible only to the current user

## Configuration

Set the credential store type in your environment file:

```env
# .env.local
CREDENTIAL_STORE=env                    # For development
CREDENTIAL_STORE=windows_credential_manager  # For production
```

## Usage

### Setting Credentials

#### Development (.env.local)
```env
BROKER_API_KEY=your_api_key_here
BROKER_API_SECRET=your_api_secret_here
BROKER_ACCOUNT_ID=your_account_id
DATA_PROVIDER_API_KEY=data_api_key
DATA_PROVIDER_ENDPOINT=https://api.dataprovider.com
```

#### Production (Windows Credential Manager)
Will be set through the application UI or command-line tools.

### Accessing Credentials in Code

```rust
use crate::credentials::CredentialsManager;

let mut creds = CredentialsManager::new()?;

// Get broker credentials
if let Some(broker) = creds.get_broker_credentials()? {
    println!("API Key: {}", broker.api_key);
    // Use credentials...
}

// Get individual credential
if let Some(value) = creds.get_credential("CUSTOM_KEY")? {
    // Use credential...
}
```

## Security Best Practices

1. **Never commit credentials to version control**
   - Add `.env.local` to `.gitignore`
   - Use GitHub Secrets for CI/CD

2. **Use appropriate storage backend**
   - Environment variables for development only
   - Windows Credential Manager for production

3. **Rotate credentials regularly**
   - Update API keys periodically
   - Revoke old credentials after rotation

4. **Validate credentials**
   - Check for empty or invalid values
   - Verify API connectivity on startup

5. **Limit credential scope**
   - Use read-only API keys where possible
   - Create separate keys for different environments

## Troubleshooting

### Credentials not found
- Check environment variable names match exactly
- Verify `.env.local` file is in project root
- Ensure environment file is loaded on startup

### Windows Credential Manager issues
- Run application with appropriate permissions
- Check Windows Credential Manager UI for stored credentials
- Verify Windows service is running

### Environment variable conflicts
- Check for system-wide environment variables
- `.env.local` takes precedence over `.env.development`
- Restart application after changing credentials

## CI/CD Configuration

For GitHub Actions, add secrets in repository settings:

1. Go to Settings → Secrets and variables → Actions
2. Add repository secrets:
   - `BROKER_API_KEY`
   - `BROKER_API_SECRET`
   - `DATA_PROVIDER_API_KEY`

Access in workflow:
```yaml
env:
  BROKER_API_KEY: ${{ secrets.BROKER_API_KEY }}
  BROKER_API_SECRET: ${{ secrets.BROKER_API_SECRET }}
```