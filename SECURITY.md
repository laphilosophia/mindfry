# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to: **erdemarslan@ymail.com**

### What to Include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Resolution Target**: Within 30 days (depending on severity)

### What to Expect

1. **Acknowledgment**: We will confirm receipt of your report
2. **Investigation**: We will investigate and validate the issue
3. **Communication**: We will keep you informed of our progress
4. **Credit**: We will credit you in security advisories (unless you prefer anonymity)

## Security Best Practices

When deploying MindFry:

1. **Network Isolation**: Run on internal networks only; do not expose to public internet without authentication proxy
2. **Firewall Rules**: Restrict port 9527 access to trusted clients only
3. **Resource Limits**: Configure `maxFrameSize` to prevent memory exhaustion attacks
4. **Monitoring**: Enable logging and monitor for unusual connection patterns

## Known Limitations

- **No Built-in Authentication**: MFBP protocol does not include authentication. Use network-level security.
- **No Encryption**: TCP traffic is unencrypted. Use TLS termination proxy for production.

## Security Advisories

Published advisories will be listed here and on the GitHub Security tab.
