# cyberdan-c4-solver

A solver for the board game Connect Four.

Built in Rust, the engine uses negamax search with alpha-beta pruning over a
bitboard representation to find strong moves at configurable difficulty levels.
Play interactively on the command line or integrate with the REST API.


## Features

### For Players

- Interactive CLI with Human vs Human and Human vs Computer modes
- Three difficulty levels: Easy, Normal, Hard (search depths 3, 9, 18)
- Configurable solver timeout
- Colorized terminal output with ANSI colors

### For Developers

- REST API for programmatic access (Axum-based)
- Bitboard representation using a `position`/`mask` u64 pair
- Iterative deepening with timeout support
- Center-first static move ordering
- Docker support for the API server
- Test suite covering win detection, move validation, and solver behavior


## Quick Start

```bash
git clone https://github.com/cyberdan/cyberdan-c4-solver.git
cd cyberdan-c4-solver
cargo build --release
./target/release/c4-solver
```

With Docker (API server):

```bash
docker build -t c4-api . && docker run -p 3000:3000 c4-api
```


## Installation and Build

### Prerequisites

- [Rust toolchain](https://rustup.rs/) (stable)

### Build the CLI

```bash
cargo build --release
```

The binary is written to `target/release/c4-solver`.

### Build the API server

```bash
cargo build --release --features api --bin c4-api
```

The binary is written to `target/release/c4-api`.

### Debug builds

Omit `--release` for faster compilation during development:

```bash
cargo build              # CLI
cargo build --features api --bin c4-api  # API
```


## Usage

### CLI

```bash
c4-solver [--timeout <seconds>]
```

Options:

- `--timeout <seconds>` -- Solver time limit per move (default: 5.0)
- `-h, --help` -- Show help message

#### Game flow

1. Select mode: Human vs Human or Human vs Computer
2. If playing against the computer, select difficulty (Easy / Normal / Hard)
3. Choose your color: Red (plays first) or Yellow (plays second)
4. Enter a column number 1-7 on your turn to drop a piece

### REST API

Start the server:

```bash
./target/release/c4-api
```

The server listens on `0.0.0.0:3000` by default.

#### Environment variables

| Variable          | Description                                | Default              |
|-------------------|--------------------------------------------|----------------------|
| `PORT`            | Port to listen on                          | `3000`               |
| `CORS_PERMISSIVE` | Set to any value to allow all CORS origins | unset (restrictive)  |
| `RUST_LOG`        | Log level (e.g. `info`, `debug`)           | unset                |

#### Endpoints

**`GET /health`**

Returns `"ok"`. Use for liveness checks.

**`POST /solve`**

Accepts a board state and returns the best move.

Request body:

```json
{
  "moves": [3, 2, 3],
  "difficulty": "normal",
  "timeout": 5.0
}
```

| Field        | Type       | Required | Description                                          |
|--------------|------------|----------|------------------------------------------------------|
| `moves`      | `number[]` | yes      | Sequence of 0-indexed columns (0-6) played so far    |
| `difficulty` | `string`   | no       | `"easy"`, `"normal"`, or `"hard"` (default: `normal`) |
| `timeout`    | `number`   | no       | Solver time limit in seconds, 0.1-10.0 (default: 5.0) |

Response body:

```json
{
  "column": 3,
  "status": "ongoing"
}
```

| Field    | Type     | Description                                          |
|----------|----------|------------------------------------------------------|
| `column` | `number` | 0-indexed column for the best move                   |
| `status` | `string` | Game state after the move: `"win"`, `"draw"`, or `"ongoing"` |

Error response:

```json
{
  "error": "move 2: column 3 is full"
}
```

Errors return HTTP 400 with a JSON object containing an `error` string.

#### Example

```bash
curl -X POST http://localhost:3000/solve \
  -H "Content-Type: application/json" \
  -d '{"moves": [3, 2, 3], "difficulty": "hard", "timeout": 5.0}'
```


## Architecture

### Bitboard

The board is stored as two `u64` values: `position` (current player's pieces) and
`mask` (all pieces). The layout is column-major with 7 bits per column (6 playable
rows plus 1 sentinel bit), fitting the full 7x6 board in a single 64-bit integer.

Win detection uses bit-shift tests in four directions (horizontal, vertical, and
both diagonals). Each direction requires two shifts and two AND operations,
checking all four-in-a-row alignments simultaneously.

### Solver

The search uses negamax with alpha-beta pruning, wrapped in iterative deepening
from depth 1 up to the difficulty's maximum depth. If the timeout is reached
mid-search, the incomplete depth is discarded and the result from the last fully
completed depth is used. The clock is checked every 1024 nodes to amortize the
cost of `Instant::now()`.

### Evaluation

Leaf nodes are scored with a positional weight heuristic. Each cell has a static
weight approximating how many four-in-a-row lines pass through it, with center
cells weighted highest (center column bottom: 7, corners: 3). The score is the
difference between the current player's total weight and the opponent's.

### Move ordering

Columns are explored in static center-first order: `[3, 2, 4, 1, 5, 0, 6]`.
This improves alpha-beta cutoffs by searching the most likely strong moves first.

### Module map

| File              | Purpose                                        |
|-------------------|------------------------------------------------|
| `src/bitboard.rs` | Board representation, move logic, win detection |
| `src/solver.rs`   | Negamax search, evaluation, iterative deepening |
| `src/game.rs`     | Game loop coordinating two player controllers   |
| `src/player.rs`   | Human and computer player implementations       |
| `src/display.rs`  | Terminal output with ANSI colors                |
| `src/main.rs`     | CLI entry point, argument parsing, menu flow    |
| `src/bin/api.rs`  | REST API server (Axum)                          |
| `src/lib.rs`      | Library crate root, module exports              |


## Docker

Build the image:

```bash
docker build -t c4-api .
```

Run the container:

```bash
docker run -p 3000:3000 c4-api
```

Configure with environment variables:

```bash
docker run -p 8080:8080 -e PORT=8080 -e CORS_PERMISSIVE=1 -e RUST_LOG=info c4-api
```


## Development

### With Nix

The project includes a Nix flake with a complete development environment:

```bash
nix develop
```

Or with direnv:

```bash
direnv allow
```

The flake provides the Rust stable toolchain (via fenix), rust-analyzer,
cargo-watch, cargo-deny, cargo-edit, and other tools.

### Without Nix

Install the Rust toolchain with [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Dev workflow

Run tests:

```bash
cargo test
```

Watch for changes and re-run tests:

```bash
cargo watch -x test
```

Lint:

```bash
cargo clippy --all-targets --all-features
```

Format:

```bash
cargo fmt
```

Enable debug logging for the solver:

```bash
RUST_LOG=debug cargo run
```

### Project structure

```
cyberdan-c4-solver/
  src/
    bin/
      api.rs          # REST API server
    bitboard.rs       # Board representation
    display.rs        # Terminal output
    game.rs           # Game loop
    lib.rs            # Library crate root
    main.rs           # CLI entry point
    player.rs         # Player controllers
    solver.rs         # Search engine
  Cargo.toml
  Dockerfile
  flake.nix
  LICENSE
```


## Testing

Run the full test suite:

```bash
cargo test
```

The suite includes 14 bitboard tests and 4 solver tests covering:

- Win detection in all four directions (horizontal, vertical, both diagonals)
- Column-full and out-of-range error handling
- Draw detection on a full board
- No false positives on empty boards
- ASCII board parsing and round-trip consistency
- Solver picks immediate wins and blocks opponent wins
- Empty board evaluates to zero
- Center column preference on an empty board
