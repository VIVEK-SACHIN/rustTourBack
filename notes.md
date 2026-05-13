# Rust Backend Quick Notes (for Node.js Developers)

## 1) Install Rust

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Verify installation

```bash
rustc --version
cargo --version
```

---

## 2) Create and Run a New Project

```bash
cargo new rust_api
cd rust_api
cargo run
```

---

## 3) Basic Rust Project Structure

```txt
rust_api/
├── Cargo.toml
├── Cargo.lock
├── src/
│   └── main.rs
└── target/
```

- `Cargo.toml`: project metadata + dependencies (similar to `package.json`)
- `Cargo.lock`: exact dependency versions (like `package-lock.json`)
- `src/main.rs`: app entry point (like `index.js` / `server.js`)
- `target/`: build output folder

---

## 4) Add Dependencies

### Option A: edit `Cargo.toml` manually

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

Then run:

```bash
cargo build
```

### Option B: use Cargo commands

```bash
cargo add axum
cargo add tokio --features full
cargo add serde --features derive
cargo add serde_json
```

---

## 5) Useful Cargo Commands

```bash
cargo build           # Build project
cargo build --release # Production build
cargo run             # Build + run
cargo check           # Fast compile check (no binary output)
cargo update          # Update dependency versions from Cargo.toml
cargo clean           # Remove build cache (target/)
```

---

## 6) Rust vs Node.js Quick Mapping

| Node.js | Rust |
| --- | --- |
| Express | Axum |
| npm | Cargo |
| package.json | Cargo.toml |
| Promise | Future |
| JS object | Struct |
| Garbage collection | Ownership + borrowing |
| Dynamic typing | Static typing |

---

## 7) Suggested Backend Structure (Rust)

```txt
my_backend/
├── Cargo.toml
├── .env
├── src/
│   ├── main.rs
│   ├── config/
│   │   └── mod.rs
│   ├── routes/
│   │   ├── mod.rs
│   │   └── user_routes.rs
│   ├── handlers/
│   │   └── user_handler.rs
│   ├── services/
│   │   └── user_service.rs
│   ├── models/
│   │   └── user.rs
│   ├── db/
│   │   └── mod.rs
│   ├── middleware/
│   │   └── auth.rs
│   └── utils/
│       └── logger.rs
└── target/
```