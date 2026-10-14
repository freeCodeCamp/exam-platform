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

## aAPI → mDb

```mermaid
graph LR
  aAPI["aAPI"] -->|"aAPI DB user; authZ read/write candidates, attempts, responses, evidence"| mDb[("mDb")]
```

## eDd → mDb

```mermaid
graph LR
  eDd["eDd"] -->|"eDd DB user; authZ read candidate data, write moderation outcomes only"| mDb[("mDb")]
```

## eDd → eDb

```mermaid
graph LR
  eDd["eDd"] -->|"eDd DB user; authZ full access, sole user"| eDb[("eDb")]
```
