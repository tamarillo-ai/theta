# Security Review Checklist

## Input Validation
- [ ] All user input sanitized before database queries
- [ ] File upload MIME types validated
- [ ] Path traversal prevented on file operations
- [ ] Command injection blocked (no unsanitized input in subprocess)
- [ ] Request body size limits enforced

## Authentication
- [ ] JWT tokens expire within 24 hours
- [ ] Refresh tokens use rotation with family tracking
- [ ] API keys stored in environment variables, not source
- [ ] Passwords hashed with bcrypt or argon2
- [ ] Rate limiting on authentication endpoints

## Authorization
- [ ] Role-based access control on sensitive endpoints
- [ ] Resource ownership verified before mutations
- [ ] Admin endpoints restricted to admin roles

## Data Protection
- [ ] Sensitive data not logged (passwords, tokens, keys)
- [ ] HTTPS enforced for all external communication
- [ ] PII handled according to retention policies

## Dependencies
- [ ] No known CVEs in direct dependencies
- [ ] Dependencies pinned to specific versions
- [ ] Bandit security scan passes

## Project-Specific
- [ ] CLI adapters use CLICommunicator, not raw subprocess
- [ ] No imports between orchestrator/ and agentic_team/
- [ ] Agent failures return AgentResponse(success=False), never raise
- [ ] Health checks use shutil.which(), not subprocess which
