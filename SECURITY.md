# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability in Tempus Engine, please report it responsibly.

**Please do NOT disclose security issues publicly via GitHub issues.**

Report through one of these channels:
1. **Email**: security@tempus-engine.dev (preferred)
2. **GitHub Security Advisories**: use the "Report a vulnerability" feature

## What to Include

- **Description**: clear description of the issue
- **Impact**: what an attacker can do
- **Reproduction steps**: deterministic steps or PoC
- **Environment**: version, OS, configuration
- **Proposed fix** (optional)

## Response & Remediation SLA

- **Acknowledgment**: within **48 hours**
- **Triage and severity assignment**: within **5 business days**
- **Status updates**: at least every **7 calendar days** while open

Target remediation windows:
- **Critical**: 7 days
- **High**: 30 days
- **Medium**: 90 days
- **Low**: next scheduled release (or earlier)

## Disclosure Policy

- We follow coordinated disclosure.
- We will publish a security advisory after a fix is available, with mitigation guidance.
- We will credit reporters who want to be acknowledged.

## Security Best Practices

### API Keys
- Store API keys in a secret manager
- Rotate keys regularly
- Never commit keys to version control

### Database
- Use strong PostgreSQL passwords
- Enable SSL/TLS for database connections
- Maintain encrypted backups

### Network
- Run behind a reverse proxy (nginx/traefik)
- Use HTTPS in production
- Implement rate limiting

### Secrets
```bash
# Never do this:
SECRET_KEY="hardcoded-secret"

# Do this instead:
SECRET_KEY=$(openssl rand -hex 32)
```

## Security Features

Tempus Engine implements:
- API Key authentication with tenant isolation
- Cryptographic hashing of pricing rules (SHA-256)
- Immutable audit trail via PostgreSQL constraints
- Input validation via JSON Schema
- Time-range constraints preventing overlapping rule versions

## Third-Party Dependencies

We monitor dependencies for known vulnerabilities:

```bash
uv run pip-audit
cd tempus_core && cargo audit
```

## Hall of Fame

We thank security researchers who responsibly disclose vulnerabilities.

*None yet — be the first!*

---

Thank you for helping keep Tempus Engine secure.
