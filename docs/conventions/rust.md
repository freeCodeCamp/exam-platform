# Rust

## Structure

For a server with REST API and static file server.

```
app/
| client/
| dist/     # Build of client
| server/
  | routes/   # Add all routes to their own module here
    | mod.rs  # Export modules from here
  | app.rs    # App router
  | config.rs # Config (mostly env vars) + server state unique to this app
  | main.rs   # In charge of tooling setup + teardown
| Cargo.toml
```

`app.rs` is a module that constructs the application router, not a separate library crate. The binary entrypoint `main.rs` loads configuration, initializes telemetry, and starts the runtime through shared serving helpers.

New routes are added as modules to `server/routes/`
