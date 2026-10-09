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

## aAPI → cAPI

```mermaid
graph LR
  aAPI["aAPI"] -->|"mTLS, client cert identifies aAPI; authZ read-only redacted content"| cAPI["cAPI"]
```

## eDd → cAPI

```mermaid
graph LR
  eDd["eDd"] -->|"mTLS, client cert identifies eDd; authZ trigger fetch, fast-forward and promotion on behalf of examiner role"| cAPI["cAPI"]
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

## eDd → Authoring clone

```mermaid
graph LR
  eDd["eDd"] -->|"eDd Dolt user; authZ read/write authoring branches"| clone[("Authoring clone")]
```

## cAPI → cDb

```mermaid
graph LR
  cAPI["cAPI"] -->|"cAPI Dolt user, sole cDb credential holder; authZ sole writer"| cDb[("cDb")]
```

## cDb → Authoring clone (fetch, driven by cAPI)

```mermaid
graph LR
  cDb[("cDb")] -->|"TLS, remotesapi fetch-only user on clone; authZ read authored history"| clone[("Authoring clone")]
```

## Authoring clone → cDb (clone and schema pull)

```mermaid
graph LR
  clone[("Authoring clone")] -->|"TLS, remotesapi read-only user on cDb; authZ clone and pull, no push"| cDb[("cDb")]
```
