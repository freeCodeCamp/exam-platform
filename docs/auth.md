# Auth

All service-to-service traffic stays on the private `platform` network. Each database grants per-service users with least privilege.

## Examiner → eDd

```mermaid
graph LR
  examiner(["Examiner"]) -->|"OIDC SSO login, session cookie; authZ by roles and scopes in eDb, enforced by eDd server"| eDd["eDd"]
```

## dApp → aAPI

```mermaid
graph LR
  dApp["dApp"] -->|"Candidate login, short-lived bearer session token; authZ scoped to candidate's own attempt"| aAPI["aAPI"]
```

## wApp → aAPI

```mermaid
graph LR
  wApp["wApp"] -->|"One-time pairing code from dApp attempt, short-lived bearer session token; authZ scoped to paired attempt"| aAPI["aAPI"]
```

## aAPI → eDb

```mermaid
graph LR
  aAPI["aAPI"] -->|"DB role aapi: no DDL; authZ read/write candidates, attempts, responses, evidence"| eDb[("eDb")]
```

## eDd → eDb

```mermaid
graph LR
  eDd["eDd"] -->|"DB role edd: no DDL; authZ examiner identity, sessions, moderation outcomes; read candidate data"| eDb[("eDb")]
```
