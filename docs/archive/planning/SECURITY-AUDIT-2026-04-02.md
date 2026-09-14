# CRONUS Security Audit — 2026-04-02

## 12 Vulnerabilities Found

### CRITICAL (2)
1. **Stored XSS** — DB values rendered without HTML escaping in data_table.rs + ui.rs
2. **SQL Injection** — binding.rs aggregation uses string interpolation

### HIGH (4)
3. **SQL Injection** — column names from JSON body used directly in INSERT/UPDATE
4. **Privilege Escalation** — signup accepts `role: "admin"` from request body
5. **No Auth on Mutations** — POST/PATCH/DELETE endpoints completely unauthenticated
6. **No CSRF** — cookies missing SameSite/Secure + CORS wildcard

### MEDIUM (4)
7. **DoS** — login/signup loads entire user table into memory
8. **Weak JWT Secret** — DefaultHasher (SipHash) not cryptographically secure
9. **Path Traversal** — `source` page config reads arbitrary files
10. **No Rate Limiting** — auth endpoints can be brute-forced

### LOW (2)
11. **Wildcard CORS** — default `*` allows any origin
12. **No Body Size Limit** — main.rs reads unlimited POST bodies

## Security Stack (from crypto research)
- Argon2id for passwords
- JWT httpOnly + SameSite=Strict + Secure cookies
- Token bucket rate limiting (in-memory)
- CSP with per-request nonces
- 12 security headers on every response
- HTML escaping on all output
- Column name validation (alphanumeric + underscore only)
- Parameterized queries everywhere (no string interpolation)
