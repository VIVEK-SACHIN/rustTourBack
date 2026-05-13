# Rust API Crates Explanation

## axum

Web framework for building APIs and web servers.

Features:
- Routing
- HTTP request handling
- JSON responses
- Middleware
- WebSockets

Example:

```rust
.route("/hello", get(handler))
```

---

## tokio

Async runtime for Rust.

Handles:
- Async execution
- Networking
- Concurrency
- Task scheduling

Example:

```rust
#[tokio::main]
async fn main() {}
```

---

## serde

Serialization/deserialization framework.

Converts:
- Rust structs → JSON
- JSON → Rust structs

Example:

```rust
#[derive(Serialize, Deserialize)]
```

---

## serde_json

JSON implementation for serde.

Functions:
- Parse JSON
- Generate JSON

Example:

```rust
serde_json::to_string(&data)
```

---

# Dependency Relationship

```text
axum
 └── uses tokio
 └── uses serde
      └── uses serde_json
```

---

# Typical API Flow

Client Request
↓
Axum Route
↓
Tokio Runtime
↓
Serde Parses JSON
↓
Business Logic
↓
Serde Converts Response
↓
Client Response