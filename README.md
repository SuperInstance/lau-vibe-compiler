# lau-vibe-compiler

**The vibe-to-code compiler — natural language compiles deterministically to PLATO operations.**

Speak your intent in plain English. The compiler tokenizes, parses, and emits a sequence of PLATO ops — the instruction set that drives the Lau platform. "I need 3 rooms" becomes `CreateRoom × 3`. "Build a fast agent" becomes `CreateAgent { archetype: "fast" }`. No LLM in the loop. Pure deterministic compilation.

---

## What This Does

`lau-vibe-compiler` is a three-stage compiler pipeline:

1. **Lex** (`VibeLexer`): tokenizes natural language into typed tokens — rooms, agents, hardware, bridges, skills, traditions, actions, quantities, modifiers, emotions.
2. **Parse** (`VibeParser`): constructs an abstract syntax tree (`VibeAST`) from the token stream. Handles create, modify, destroy, query, deploy, reset, load, and test commands.
3. **Compile** (`VibeCompiler`): emits `PlatoOp` instructions — the IR that the PLATO runtime executes.

The whole thing runs in microseconds, no GPU required. Think of it as a domain-specific language where the syntax is English and the semantics are PLATO.

---

## Key Idea

The compiler treats natural language as a **regular language** over a fixed vocabulary. By mapping words to token types (Room, Agent, Hardware, Bridge, Skill, Tradition, Action, Quantity, Modifier, Emotion) and using pattern-matching rules on the token stream, it achieves deterministic parsing without ambiguity resolution or probabilistic models.

The output — `PlatoOp` — is a typed intermediate representation. Every vibe command compiles to a known set of operations. Programs can be saved, loaded, serialized, and executed.

---

## Install

```toml
[dependencies]
lau-vibe-compiler = "0.1.0"
```

```bash
cargo add lau-vibe-compiler
```

Requires Rust 2021 edition. Dependencies: `serde`, `serde_json`.

---

## Quick Start

```rust
use lau_vibe_compiler::*;

let compiler = VibeCompiler::new();

// Create rooms
let ops = compiler.compile("I need 3 rooms").unwrap();
// → [CreateRoom { name: "room_1", ... }, CreateRoom { name: "room_2", ... }, CreateRoom { name: "room_3", ... }]

// Create an agent with tradition training
let ops = compiler.compile("I need an agent to learn african symmetry").unwrap();
// → [CreateAgent { archetype: "default", skills: ["symmetry"] }, TrainAgent { tradition: Some("african") }]

// Modify, destroy, query
let ops = compiler.compile("make the room big").unwrap();
// → [ModifyItem { id: "room", property: "modifier", value: "big" }]

let ops = compiler.compile("remove the agent").unwrap();
// → [DestroyItem { id: "agent" }]

let ops = compiler.compile("how is the room").unwrap();
// → [QueryStatus { target: "room" }]

// System commands
let ops = compiler.compile("deploy").unwrap();
// → [DeployAll]

let ops = compiler.compile("load combat program").unwrap();
// → [LoadProgram { name: "combat" }]
```

---

## API Reference

### VibeToken

```rust
pub struct VibeToken {
    pub text: String,
    pub token_type: VibeTokenType,
    pub confidence: f64,
}

pub enum VibeTokenType {
    Room, Agent, Hardware, Bridge, Skill, Tradition,
    Action, Modifier, Quantity, Emotion, Raw,
}
```

### VibeLexer

```rust
let tokens = VibeLexer::lex("I need a big room for 3 agents");
// → [Raw("i"), Action("need"), Raw("a"), Modifier("big"), Room("room"), Raw("for"), Quantity("3"), Agent("agents")]
```

**Recognized keyword categories:**

| Category | Examples |
|----------|---------|
| Room | room, space, chamber, zone, deck, lab |
| Agent | agent, crew, worker, bot, specialist, archetype |
| Hardware | motor, servo, sensor, gpio, esp32, controller |
| Bridge | bridge, connection, link, tunnel, portal |
| Skill | train, learn, teach, practice, drill, program |
| Tradition | greek, chinese, taoist, vedic, islamic, japanese, african, adinkra, quipu, songline |
| Action | build, create, make, deploy, connect, control, remove, destroy, delete, reset, test, load, need, want, give, set, change, how |
| Quantity | three–ten, lots, many, several, all, some, numeric strings |
| Modifier | big, small, fast, slow, safe, dangerous, heavy, light, bright, dark, warm, cold |
| Emotion | happy, angry, calm, excited, fearful, brave, gentle, fierce, peaceful, chaotic |

### VibeAST

```rust
pub enum VibeAST {
    Create(CreateSpec),
    Modify(String, ModifySpec),
    Destroy(String),
    Query(QuerySpec),
    Deploy,
    Reset,
    LoadProgram(String),
    Test(String),
}
```

### VibeCompiler

```rust
let compiler = VibeCompiler::new();

// Full pipeline: lex → parse → compile
let ops: Result<Vec<PlatoOp>, String> = compiler.compile("build a bridge");

// Individual stages
let tokens = compiler.lex("I need a room");
let ast = compiler.parse(tokens)?;
let ops = compiler.compile_ast(ast);

// Batch
let results: Vec<Result<Vec<PlatoOp>, String>> = compiler.batch_compile(vec!["deploy", "reset"]);
```

### PlatoOp

```rust
pub enum PlatoOp {
    CreateRoom { name, room_type, properties },
    CreateAgent { name, archetype, skills },
    CreateHardware { name, hw_type, channels },
    CreateBridge { target },
    TrainAgent { agent, skill, tradition },
    ModifyItem { id, property, value },
    DestroyItem { id },
    DeployAll,
    ResetAll,
    QueryStatus { target },
    TestItem { id },
    LoadProgram { name },
}
```

### VibeProgram

Pre-packaged sequences of PLATO ops, like Neo loading combat training:

```rust
let program = combat_program();
let result = program.execute();
// ProgramResult { ops_executed: 3, success: true, message: "..." }
```

**Built-in programs:**

| Program | Description |
|---------|-------------|
| `combat_program()` | Training hall + warrior agent + Greek combat training |
| `engineering_program()` | Workshop + motor array + mechanic agent |
| `exploration_program()` | Scout agent + map room + terrain skills |
| `social_program()` | Palaver room + diplomat agent + faction bridges |

### VoiceAnnotation

Extra metadata from voice input (prosody, confidence, language). Used to annotate compiler input with emotional context:

```rust
let va = VoiceAnnotation::from_text("hello world");
// va.confidence == 1.0, va.prosody == None
```

---

## How It Works

### Lexing

The lexer normalizes input to lowercase, splits on whitespace, then classifies each word against fixed keyword sets. It also handles bigrams ("lots of", "a bunch of") before single-word classification. Unrecognized words become `Raw` tokens. Numeric strings parse as `Quantity`.

### Parsing

The parser inspects the **token type sequence** to determine the command:

1. **Deploy/Reset**: detected by action keywords "deploy" or "reset"
2. **LoadProgram**: triggered by "load" followed by a name
3. **Test**: triggered by "test" followed by a target
4. **Destroy**: triggered by "remove"/"destroy"/"delete" followed by a target
5. **Query**: triggered by "how" followed by a target
6. **Modify**: triggered by "make"/"set"/"change" followed by target + value
7. **Create**: triggered by "need"/"want"/"create"/"build"/"make"/"give" followed by a typed item
8. **Fallback**: looks for any item type token, or uses raw tokens as a freeform name

### Compilation

Each AST node maps to one or more `PlatoOp`:

- **Create** with quantity > 1 generates multiple ops with unique IDs (via an internal counter)
- **Create Agent** with a tradition property also emits a `TrainAgent` op
- Item types are inferred from the first recognized type token
- Properties (tradition, modifier, emotion, curriculum) are extracted from their respective token types

---

## The Math

This is a **deterministic compiler** — there is no probabilistic model. The mapping from input to output is a pure function:

$$\text{compile}: \text{String} \xrightarrow{\text{lex}} [\text{Token}] \xrightarrow{\text{parse}} \text{AST} \xrightarrow{\text{codegen}} [\text{PlatoOp}]$$

Each stage is total (never crashes) and deterministic (same input → same output). The parser uses ordered pattern-matching rules, not backtracking or ambiguity resolution. Compilation time is O(n) in input length.

The internal counter for unique IDs uses `Cell<u64>` — monotonically increasing, no wrapping.

---

## Test Coverage

**55 tests** covering:

- **VibeToken**: construction, serialization
- **VibeLexer**: all 11 token types, bigrams, empty input, multiple traditions
- **VibeParser**: all 8 AST variants, quantity extraction, empty input error
- **VibeCompiler**: end-to-end compilation for all operation types, batch compile, multiple quantities, default trait
- **VibeProgram**: new, add_op, execute, empty execute, serialization
- **Pre-built programs**: combat, engineering, exploration, social — all verify op count and success
- **VoiceAnnotation**: from_text, serialization, prosody
- **Integration**: create room with tradition, fast agent, bridge connection, PlatoOp serialization roundtrip

```bash
cargo test
```

---

## License

MIT
