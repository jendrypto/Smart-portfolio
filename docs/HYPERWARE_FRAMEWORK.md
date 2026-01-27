# Hyperware Development Framework Reference

Reference document for the Hyperware platform tools and libraries used to build Smart Portfolio.

---

## Table of Contents

- [Overview](#overview)
- [Kit CLI Toolkit](#kit-cli-toolkit)
- [Process Library (hyperware_process_lib)](#process-library-hyperware_process_lib)
- [Core Concepts](#core-concepts)
- [Development Workflow](#development-workflow)
- [Package Structure](#package-structure)
- [Process Architecture](#process-architecture)
- [HTTP Server & Client](#http-server--client)
- [State Management](#state-management)
- [Capabilities & Security](#capabilities--security)
- [Testing](#testing)
- [Publishing](#publishing)
- [External Resources](#external-resources)

---

## Overview

Hyperware is a platform for building local-first, privacy-preserving applications that run as WebAssembly processes on personal nodes. The development stack consists of two primary components:

| Component | Purpose | Version |
|-----------|---------|---------|
| **[kit](https://github.com/hyperware-ai/kit)** | CLI development toolkit | Latest via cargo install |
| **[hyperware_process_lib](https://github.com/hyperware-ai/process_lib)** | Rust standard library for processes | 3.0.0 |

Processes are written in Rust (primary), compiled to `wasm32-wasip1`, and deployed to Hyperware nodes. Frontends are built with standard web technologies (React, Vite, etc.) and served by the process via the built-in HTTP server.

---

## Kit CLI Toolkit

### Installation

```bash
cargo install --git https://github.com/hyperware-ai/kit --locked
```

Update with `kit update` or re-run the install command.

**Requirements:** Rust toolchain, Node.js 18+, npm.

### Commands Reference

| Command | Short | Description |
|---------|-------|-------------|
| `kit new` | `kit n` | Create a new package from template |
| `kit build` | `kit b` | Compile package to WASM + bundle UI |
| `kit start-package` | `kit s` | Deploy package to a running node |
| `kit build-start-package` | — | Combined build and deploy |
| `kit boot-fake-node` | `kit f` | Start a local development node |
| `kit boot-real-node` | — | Launch a production node |
| `kit dev-ui` | `kit d` | Hot-reloading UI development server |
| `kit inject-message` | `kit i` | Send test messages to a process |
| `kit run-tests` | `kit t` | Execute test suites from TOML config |
| `kit publish` | — | Publish package to the Hypermap |
| `kit chain` | `kit c` | Start a local fakechain (anvil) |
| `kit connect` | — | SSH tunnel to remote nodes |
| `kit remove-package` | — | Uninstall a package |
| `kit reset-cache` | — | Clear cached data |
| `kit view-api` | `kit v` | Display package WIT APIs |

### kit new

Creates a package from a template.

```bash
kit new my-app              # Rust chat template, no UI
kit new my-app --ui         # Rust chat template with UI
kit new my-app -t echo      # Echo template
kit new my-app -t fibonacci # Fibonacci template
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--package` | `-a` | Package identifier (Hypermap-safe) | Directory name |
| `--publisher` | `-u` | Publisher identifier | `template.os` |
| `--language` | `-l` | Language (`rust`) | `rust` |
| `--template` | `-t` | Template (`chat`, `echo`, `fibonacci`, `file-transfer`) | `chat` |
| `--ui` | — | Include UI scaffolding | Disabled |

Package names must be **Hypermap-safe**: lowercase `a-z`, digits `0-9`, and hyphens only.

### kit build

Compiles processes to WASM and bundles the UI.

```bash
kit build              # Build current directory
kit build my-app       # Build specific package
```

| Option | Short | Description |
|--------|-------|-------------|
| `--no-ui` | — | Skip UI compilation |
| `--ui-only` | — | Build only the UI |
| `-i, --include` | — | Build specific processes only |
| `-e, --exclude` | — | Exclude specific processes |
| `--features` | — | Pass Rust cargo feature flags |
| `-r, --reproducible` | — | Deterministic Docker-based build |
| `-f, --force` | — | Rebuild regardless of cache |
| `--hyperapp` | — | Build as Hyperapp framework app |

The build process:
1. Detects process language (Rust/Python/JS) from `src/lib.*`
2. Compiles each process to `.wasm`
3. Runs `npm install && npm run build:copy` for UI (if `ui/` exists)
4. Places all outputs in `pkg/`

### kit boot-fake-node

Starts a local development node disconnected from the live network.

```bash
kit boot-fake-node                              # Default: port 8080, node fake.os
kit boot-fake-node -p 8081 -f fake2.os          # Second node for multi-node testing
kit boot-fake-node --persist                    # Keep state between runs
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--port` | `-p` | Node HTTP port | 8080 |
| `--fake-node-name` | `-f` | Node identifier | `fake.os` |
| `--home` | `-o` | Home directory | `/tmp/hyperware-fake-node` |
| `--fakechain-port` | `-c` | Anvil chain port | 8545 |
| `--persist` | — | Retain state after shutdown | Off |
| `--password` | — | Login credential | `secret` |
| `--runtime-path` | `-r` | Build from local Hyperdrive repo | Fetch binary |
| `--version` | `-v` | Runtime version | Latest |
| `--verbosity` | — | Logging level (0-3) | 0 |
| `--release` | — | Compile optimized | Debug mode |

For multi-node testing, start nodes in separate terminals with different ports, names, and home directories.

### kit start-package

Deploys a built package to a running node.

```bash
kit start-package              # Deploy current directory to port 8080
kit start-package my-app -p 8081
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--port` | `-p` | Target node port | 8080 |

### kit dev-ui

Hot-reloading development server for UI packages.

```bash
kit dev-ui              # Serve UI from current directory
kit dev-ui my-app       # Serve specific package UI
kit dev-ui --release    # Production build mode
```

Equivalent to `cd ui && npm i && npm run dev` with automatic proxy to the running node.

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--port` | `-p` | Node port to proxy to | 8080 |
| `--release` | — | Production build | Dev mode |
| `--skip-deps-check` | `-s` | Skip npm dependency check | Off |

### kit inject-message

Send messages directly to a running process for testing.

```bash
kit inject-message my-app:my-app:template.os '{"Hello": "world"}'
kit inject-message my-app:my-app:template.os '{"Send": {"target": "fake2.os", "message": "hi"}}' -p 8080
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--port` | `-p` | Node port | 8080 |
| `--node` | `-n` | Target node | `our` |
| `--blob` | `-b` | Attach file as lazy_load_blob | — |
| `--non-block` | `-l` | Fire-and-forget (no response wait) | Blocking |

### kit run-tests

Executes test suites defined in TOML configuration.

```bash
kit run-tests                  # Run tests.toml in current directory
kit run-tests my_tests.toml    # Run specific test file
```

Tests are orchestrated externally by `kit` and executed internally by the `tester` core package. Each test:
1. Spins up fresh fake node(s)
2. Installs dependency and setup packages
3. Runs test packages sequentially
4. Reports `Pass` or `Fail` per package
5. Stops at first failure

**Test TOML structure:**
```toml
[runtime]
FetchVersion = "latest"

[[tests]]
timeout_secs = 300
fakechain_router = 8545

[[tests.nodes]]
port = 8080
home = "/tmp/test-node"
fake_node_name = "fake.os"

[tests.dependency_package_paths]
[tests.setup_packages]
[tests.test_package_paths]
```

### kit publish

Publish packages to the Hypermap onchain registry.

```bash
# Publish to fakenode
kit publish my-app --metadata-uri https://raw.githubusercontent.com/.../metadata.json \
  --rpc http://localhost:8545 --keystore-path ./key.json

# Publish to mainnet
kit publish my-app --real --metadata-uri <uri> --rpc <eth-rpc-url> --keystore-path ./key.json
```

| Option | Description |
|--------|-------------|
| `--metadata-uri` | Public URL of `metadata.json` (use commit-specific URL) |
| `--rpc` | Ethereum RPC endpoint |
| `--keystore-path` | Web3 keystore file |
| `--ledger` | Use Ledger hardware wallet |
| `--trezor` | Use Trezor hardware wallet |
| `--safe` | Generate Safe transaction calldata |
| `--real` | Deploy to mainnet (default: fakenode) |
| `--unpublish` | Remove a published package |
| `--mock` | Dry-run without submitting |
| `--gas-limit` | Gas limit (default: 1,000,000) |

### kit chain

Start a local fakechain with pre-seeded HNS and app-store contracts.

```bash
kit chain                  # Default port 8545
kit chain --port 9545      # Custom port
```

### kit connect

SSH tunnel to remote Hyperware nodes.

```bash
kit connect --host user@remote-server     # Create tunnel on port 9090
kit connect 9090 --disconnect             # Tear down tunnel
```

### kit view-api

Inspect WIT APIs exposed by packages.

```bash
kit view-api                       # List all APIs on node
kit view-api app-store:sys         # View specific package API
```

---

## Process Library (hyperware_process_lib)

The standard library crate for writing Hyperware processes in Rust.

```toml
# Cargo.toml
[dependencies]
hyperware_process_lib = "3.0.0"
wit-bindgen = "0.42.1"  # Must match process_lib version
```

**Version compatibility:**

| process_lib | wit-bindgen |
|-------------|-------------|
| 3.x.y | 0.42.1 |
| 2.x.y | 0.42.1 |
| 1.x.y | 0.36.0 |

### Module Overview

| Module | Purpose |
|--------|---------|
| `http::server` | HTTP server bindings, path routing, static file serving |
| `http::client` | Outbound HTTP requests, WebSocket connections |
| `vfs` | Virtual filesystem for file storage and UI serving |
| `kv` | Key-value store |
| `sqlite` | SQLite database interaction |
| `eth` | Ethereum provider integration |
| `net` | Networking, signatures, peer configuration |
| `timer` | Timer/scheduling runtime module |
| `homepage` | Homepage widget registration |
| `hypermap` | Onchain namespace operations |
| `sign` | Cryptographic signing |
| `dao` | DAO/Timelock/Governor contract access |
| `bindings` | Token registry contract interaction |
| `kernel_types` | Kernel-specific types |
| `scripting` | Script process types and macros |

### Core Types

```rust
use hyperware_process_lib::{
    Address,          // Process address (node + process ID)
    ProcessId,        // Process identifier
    PackageId,        // Package identifier
    Capability,       // Permission token
    Request,          // Outbound message builder
    Response,         // Response builder
    Message,          // Received message (request or response)
    LazyLoadBlob,     // Large data attachment
};
```

### Essential Functions

| Function | Purpose |
|----------|---------|
| `await_message()` | Block until next message arrives |
| `spawn()` | Create a new child process |
| `can_message()` | Check if capability exists for target |
| `get_capability()` | Retrieve a stored capability |
| `get_typed_state()` | Deserialize persistent state |
| `set_state()` | Persist serialized state |
| `get_typed_blob()` | Deserialize message blob attachment |

### Macros

| Macro | Purpose |
|-------|---------|
| `call_init!()` | WIT entry point boilerplate |
| `println!()` | Print to node terminal |
| `kiprintln!()` | Print at max verbosity |

---

## Core Concepts

### Process Identity

Every process has a globally unique address: `process-name:package-name:publisher-node`

```
smart-portfolio:smart-portfolio:template.os
```

- **process-name**: Hypermap-safe (`a-z`, `0-9`, `-`)
- **package-name**: Hypermap-safe
- **publisher-node**: Hypermap-safe with dots allowed

### Message-Passing Architecture

Processes communicate exclusively through **Request-Response messages**. There is no shared memory.

```rust
// Sending a request
Request::to(&target_address)
    .body(serde_json::to_vec(&my_data)?)
    .expects_response(5)  // 5-second timeout
    .send()?;

// Main event loop
loop {
    let message = await_message()?;
    if message.is_request() {
        // Handle incoming request
        handle_request(&message);
    } else {
        // Handle response to our earlier request
        handle_response(&message);
    }
}
```

### Serialization

Message bodies are byte vectors. Common serialization formats:

| Format | Crate | Use Case |
|--------|-------|----------|
| JSON | `serde_json` | HTTP APIs, human-readable IPC |
| Bincode | `bincode` | Compact state persistence |
| MessagePack | `rmp_serde` | Efficient IPC |

### WebAssembly Interface Types (WIT)

Processes define their APIs using WIT for language-independent interoperability:

```wit
interface my-api {
    variant request {
        hello(string),
        goodbye,
    }
    variant response {
        greeting(string),
    }
}
```

WIT files go in `api/` and are used by `wit-bindgen` to generate type bindings at compile time.

---

## Development Workflow

### Quick Start

```bash
# 1. Create project
kit new my-app --ui

# 2. Build
kit build my-app

# 3. Start dev node (Terminal 1)
kit boot-fake-node

# 4. Deploy (Terminal 2)
kit start-package my-app

# 5. Access
open http://localhost:8080/my-app:my-app:template.os/
```

### Development Loop

```bash
# Edit code, then:
kit build my-app && kit start-package my-app

# Or for UI-only changes:
kit dev-ui my-app    # Hot-reloading dev server
```

### Multi-Node Testing

```bash
# Terminal 1: First node
kit boot-fake-node

# Terminal 2: Second node
kit boot-fake-node -f fake2.os -p 8081 -o /tmp/hyperware-fake-node-2

# Terminal 3: Deploy to both
kit start-package my-app -p 8080
kit start-package my-app -p 8081
```

---

## Package Structure

```
my-package/
├── my-process/              # Rust process source
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs           # Process entry point
├── ui/                      # Frontend (optional)
│   ├── package.json
│   ├── vite.config.ts
│   └── src/
│       └── ...
├── api/                     # WIT API definitions (optional)
│   └── my-process:my-package:publisher.os-v0.wit
├── pkg/                     # Built output (generated)
│   ├── manifest.json        # Process manifest
│   ├── scripts.json         # Init scripts (optional)
│   └── *.wasm               # Compiled binaries
├── test/                    # Test suite (optional)
│   └── tests.toml
├── metadata.json            # ERC721-compatible package metadata
└── Cargo.toml               # Workspace root
```

### metadata.json

```json
{
  "name": "My App",
  "description": "App description",
  "image": "",
  "properties": {
    "package_name": "my-app",
    "current_version": "0.1.0",
    "publisher": "template.os",
    "mirrors": [],
    "code_hashes": {}
  }
}
```

`code_hashes` is populated automatically by `kit build`.

### pkg/manifest.json

Defines process startup configuration:

```json
[
  {
    "process_name": "my-app",
    "process_wasm_path": "/my-app.wasm",
    "on_exit": "Restart",
    "request_networking": false,
    "request_capabilities": [
      "http-server:distro:sys",
      "http-client:distro:sys",
      "vfs:distro:sys"
    ],
    "grant_capabilities": [],
    "public": true
  }
]
```

| Field | Description |
|-------|-------------|
| `process_name` | Process identifier |
| `process_wasm_path` | Path to compiled WASM binary in `pkg/` |
| `on_exit` | `"Restart"`, `"None"`, or JSON object for crash notifications |
| `request_networking` | Whether process needs network access |
| `request_capabilities` | System capabilities to request |
| `grant_capabilities` | Capabilities granted to other processes |
| `public` | Whether process accepts messages from any source |

---

## Process Architecture

### Entry Point Pattern

```rust
use hyperware_process_lib::{
    await_message, call_init, println, Address, Message, Request, Response,
};

wit_bindgen::generate!({
    path: "target/wit",
    world: "process-v1",
});

call_init!(init);

fn init(our: Address) {
    println!("Process started: {our}");

    // Load persisted state or create default
    let mut state: AppState = hyperware_process_lib::get_typed_state(|bytes| {
        bincode::deserialize(bytes)
    }).unwrap_or_default();

    loop {
        match await_message() {
            Ok(message) => handle_message(&our, &mut state, &message),
            Err(send_error) => println!("Error: {send_error:?}"),
        }
    }
}
```

### Process Lifecycle

- **Start**: `init()` called once when process is deployed or node restarts
- **Running**: Infinite `await_message()` loop processes requests
- **Exit behavior**: Controlled by `on_exit` in manifest
  - `"Restart"` — automatic restart (recommended for services)
  - `"None"` — process terminates permanently
  - JSON object — send notification messages to other processes

---

## HTTP Server & Client

### Server Setup

Requires `http-server:distro:sys` capability in manifest.

```rust
use hyperware_process_lib::http::server::{
    HttpServer, HttpServerRequest, HttpBindingConfig,
};

let mut server = HttpServer::new(5);

// Bind API paths (authenticated by default)
server.bind_http_path("/api", HttpBindingConfig::default()).unwrap();

// Serve static UI files from VFS
server.serve_file("ui/index.html", vec!["/"], HttpBindingConfig::default()).unwrap();
```

### Handling HTTP Requests

HTTP requests arrive as messages in the main event loop:

```rust
fn handle_message(our: &Address, state: &mut AppState, message: &Message) {
    if message.is_request() {
        if let Ok(http_request) = serde_json::from_slice::<HttpServerRequest>(message.body()) {
            match http_request {
                HttpServerRequest::Http(req) => {
                    let method = req.method();
                    let path = req.path();
                    let body = get_blob();  // Request body
                    // Route and handle...
                }
                HttpServerRequest::WebSocket(_ws) => { /* WebSocket handling */ }
            }
        }
    }
}
```

### HTTP Client (Outbound Requests)

Requires `http-client:distro:sys` capability in manifest.

```rust
use hyperware_process_lib::http::client::send_request_await_response;
use hyperware_process_lib::http::Method;

// Synchronous request with 30-second timeout
// IMPORTANT: timeout is in MILLISECONDS
let response = send_request_await_response(
    Method::GET,
    url::Url::parse("https://api.example.com/data")?,
    None,       // Optional headers
    30000,      // Timeout: 30,000 ms = 30 seconds
    vec![],     // Optional body bytes
)?;

let status = response.status();
let body = response.body();
```

**Client functions:**

| Function | Description |
|----------|-------------|
| `send_request_await_response()` | Blocking HTTP request |
| `send_request()` | Non-blocking HTTP request (response arrives in event loop) |
| `open_ws_connection()` | Open a WebSocket connection |
| `send_ws_client_push()` | Send WebSocket message |
| `close_ws_connection()` | Close WebSocket connection |

**Common pitfall:** The timeout parameter is in **milliseconds**, not seconds. Use `30000` for 30 seconds, not `30`.

---

## State Management

### Persistent State

State is serialized to bytes and stored by the Hyperware runtime. It survives process restarts and node reboots.

```rust
use hyperware_process_lib::{get_typed_state, set_state};

// Load state (returns None if no prior state)
let state: Option<AppState> = get_typed_state(|bytes| {
    bincode::deserialize(bytes)
});

// Save state
let bytes = bincode::serialize(&state).unwrap();
set_state(&bytes);
```

**Serialization notes:**
- Adding fields to a `bincode`-serialized struct will cause old state to fail deserialization. Use `unwrap_or_default()` for graceful fallback.
- `serde_json` is more tolerant of schema changes but less compact.
- State is local to the process — no shared state between processes.

### Virtual Filesystem (VFS)

For larger data or file storage, use the VFS module. Requires `vfs:distro:sys` capability.

```rust
use hyperware_process_lib::vfs;

// VFS is used internally by the HTTP server to serve UI files
// Also available for general file operations
```

### Key-Value Store

```rust
use hyperware_process_lib::kv;
// Provides key-value storage operations
```

### SQLite

```rust
use hyperware_process_lib::sqlite;
// Provides SQLite database operations
```

---

## Capabilities & Security

### Capability Model

Processes operate under a capability-based security model. A process can only communicate with another process if it holds the appropriate `Capability`.

**Requesting capabilities** (in `pkg/manifest.json`):

```json
{
  "request_capabilities": [
    "http-server:distro:sys",    // Serve HTTP endpoints
    "http-client:distro:sys",    // Make outbound HTTP requests
    "vfs:distro:sys"             // Virtual filesystem access
  ]
}
```

**Common system capabilities:**

| Capability | Purpose |
|------------|---------|
| `http-server:distro:sys` | Bind HTTP paths, serve files |
| `http-client:distro:sys` | Make outbound HTTP/WebSocket requests |
| `vfs:distro:sys` | Read/write files in virtual filesystem |
| `homepage:homepage:sys` | Register on node homepage |

**Missing capability error:** If a capability is not declared, calls to that system process will fail silently or with an error like:
```
doesn't have capability to message process http-client:distro:sys
```

### Granting Capabilities

Processes can grant access to themselves to other processes:

```json
{
  "grant_capabilities": [
    "other-process:other-package:publisher.os"
  ]
}
```

### Public Processes

Setting `"public": true` in the manifest allows any process (local or remote) to send messages to yours.

---

## Testing

### Test Architecture

Hyperware uses an integration testing approach:

1. `kit run-tests` reads a TOML configuration
2. Fresh fake node(s) are spun up per test
3. Dependencies and setup packages are installed
4. Test packages are deployed and executed by the `tester` core package
5. Each test package reports `Pass` or `Fail`

### Test Configuration (tests.toml)

```toml
[runtime]
FetchVersion = "latest"

persist_home = false
runtime_build_release = false

[[tests]]
timeout_secs = 300
fakechain_router = 8545

[[tests.nodes]]
port = 8080
home = "/tmp/test-node"
fake_node_name = "fake.os"
password = "secret"
runtime_verbosity = 0

[tests.dependency_package_paths]

[tests.setup_packages]

[tests.test_package_paths]
"my-test" = "test/my-test-process"
```

### Test Process Interface

Test packages must implement the `tester` interface, accepting `run-request` messages and responding with pass/fail results including test name, file, line, and column on failure.

### Manual Testing

```bash
# Inject messages directly
kit inject-message my-app:my-app:template.os '{"Hello": "world"}' -p 8080

# Test HTTP endpoints
curl http://localhost:8080/my-app:my-app:template.os/api/endpoint
```

---

## Publishing

### Prerequisites

1. Package built with `kit build`
2. `metadata.json` hosted publicly (use commit-specific GitHub URL)
3. Ethereum wallet (keystore file, Ledger, or Trezor)
4. ETH RPC endpoint

### Publish to Fakenode (Testing)

```bash
kit chain  # Start local chain if not running

kit publish my-app \
  --metadata-uri https://raw.githubusercontent.com/user/repo/<commit>/metadata.json \
  --rpc http://localhost:8545 \
  --keystore-path ./keystore.json
```

### Publish to Mainnet

```bash
kit publish my-app --real \
  --metadata-uri https://raw.githubusercontent.com/user/repo/<commit>/metadata.json \
  --rpc https://eth-mainnet.g.alchemy.com/v2/<key> \
  --keystore-path ./keystore.json
```

### Unpublish

```bash
kit publish my-app --unpublish --real --rpc <rpc> --keystore-path ./key.json
```

---

## External Resources

- **Hyperware Book**: https://book.hyperware.ai
- **Kit CLI Docs**: https://book.hyperware.ai/kit/kit-dev-toolkit.html
- **Process Lib API Docs**: https://docs.rs/hyperware_process_lib
- **Process Lib Crate**: https://crates.io/crates/hyperware_process_lib
- **Kit Repository**: https://github.com/hyperware-ai/kit
- **Process Lib Repository**: https://github.com/hyperware-ai/process_lib
- **Core Tests Examples**: https://github.com/hyperware-ai/core_tests
