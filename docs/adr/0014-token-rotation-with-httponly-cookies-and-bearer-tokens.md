# ADR 0014: Token Rotation with HttpOnly Cookies and Bearer Tokens

## Status
Accepted

## Context
As an Executive Operating System, Life OS must protect highly intimate personal, financial, and biometric data. In our hybrid deployment (where clients can access Life OS via standard browser views or local Tauri native desktop shells), we require an authentication architecture that guarantees top-tier security against Cross-Site Scripting (XSS) and Cross-Site Request Forgery (CSRF), while supporting seamless session persistence.

## Decision
We select a **Hybrid JWT Bearer + Secure HttpOnly Cookie Rotation** architecture:
1. **Access Token (Header)**: Standard JWT Access Tokens are issued to clients, residing entirely in-memory and passed in the `Authorization: Bearer <TOKEN>` header. These tokens expire in 15 minutes.
2. **Refresh Token (Cookie)**: Long-lived Refresh Tokens (7-day expiration) are stored inside HTTP-only, Secure, SameSite=Strict cookies, managed exclusively by the Gateway.
3. **Token Rotation**: Every access token refresh cycle automatically revokes and rotates the associated refresh token, mitigating potential session theft.
4. **Tauri Native Persistence**: Since Tauri bypasses browser cookie limitations, native installations store refresh tokens securely inside the physical host operating system's keychain (via Rust's native `keyring` library), using local IPC calls during startup.

## Rationale
- **XSS & CSRF Mitigation**: Storing the short-lived access token in memory blocks malicious scripts (XSS) from reading them. Utilizing HTTP-only and SameSite=Strict policies on refresh cookies blocks CSRF attacks and prevents client-side access.
- **Tauri Native Integration**: Using the platform's secure keychain (like Apple Keychain or Windows Credential Manager) prevents local secrets from being leaked from standard application state files on disk.
- **Observability**: Session states are validated statelessly by verifying signatures, while rotation states are tracked dynamically inside a fast Redis cache to support immediate multi-device revocation.

## Consequences
- **Positive**: Industry-grade security profile, strong protection against token hijacking, and transparent session persistence for both web and local native applications.
- **Negative**: Increases synchronization state complexity during token rotation, requiring clients to cleanly catch 401 token-expiry events and execute background retries.
