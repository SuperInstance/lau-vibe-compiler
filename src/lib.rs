use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// 1. VibeToken
// ---------------------------------------------------------------------------

/// A parsed piece of a vibe description.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VibeToken {
    pub text: String,
    pub token_type: VibeTokenType,
    pub confidence: f64,
}

/// Types of tokens the lexer can recognise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VibeTokenType {
    Room,
    Agent,
    Hardware,
    Bridge,
    Skill,
    Tradition,
    Action,
    Modifier,
    Quantity,
    Emotion,
    Raw,
}

impl VibeToken {
    pub fn new(text: &str, token_type: VibeTokenType) -> Self {
        Self {
            text: text.to_string(),
            token_type,
            confidence: 1.0,
        }
    }
}

// ---------------------------------------------------------------------------
// 2. VibeLexer
// ---------------------------------------------------------------------------

/// Tokenises natural-language vibe descriptions into typed `VibeToken`s.
pub struct VibeLexer;

impl VibeLexer {
    /// Break natural language into typed tokens.
    pub fn lex(input: &str) -> Vec<VibeToken> {
        let lower = input.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        let mut tokens = Vec::new();

        let room_kw = [
            "room", "rooms", "space", "spaces", "chamber", "area", "zone", "deck", "lab",
        ];
        let agent_kw = [
            "agent", "agents", "crew", "worker", "bot", "bots", "specialist", "archetype",
        ];
        let hw_kw = [
            "motor", "motors", "servo", "servos", "sensor", "sensors", "gpio", "esp32", "controller",
        ];
        let bridge_kw = [
            "bridge", "bridges", "connection", "link", "tunnel", "portal",
        ];
        let skill_kw = [
            "train", "learn", "teach", "practice", "drill", "program",
        ];
        let tradition_kw = [
            "greek", "chinese", "taoist", "vedic", "islamic", "japanese",
            "african", "indigenous", "adinkra", "quipu", "songline",
        ];
        let action_kw = [
            "build", "create", "make", "deploy", "connect", "control",
            "remove", "destroy", "delete", "reset", "test", "load",
            "need", "want", "give", "set", "change", "how",
        ];
        let quantity_kw = [
            "three", "four", "five", "six", "seven", "eight", "nine", "ten",
            "lots", "many", "several", "all", "some",
        ];
        let modifier_kw = [
            "big", "small", "fast", "slow", "safe", "dangerous",
            "heavy", "light", "bright", "dark", "warm", "cold",
        ];
        let emotion_kw = [
            "happy", "angry", "calm", "excited", "fearful", "brave",
            "gentle", "fierce", "peaceful", "chaotic",
        ];

        // Bigram quantities
        let bigram_quantities = ["lots of", "a bunch of"];
        let lower_full = lower.clone();
        for bq in &bigram_quantities {
            if lower_full.contains(bq) {
                tokens.push(VibeToken::new(bq, VibeTokenType::Quantity));
            }
        }

        for word in &words {
            // Skip if already captured as part of a bigram
            let w = *word;
            if w == "of" && tokens.last().is_some_and(|t| t.text == "lots") {
                continue;
            }

            if room_kw.contains(&w) {
                tokens.push(VibeToken::new(w, VibeTokenType::Room));
            } else if agent_kw.contains(&w) {
                tokens.push(VibeToken::new(w, VibeTokenType::Agent));
            } else if hw_kw.contains(&w) {
                tokens.push(VibeToken::new(w, VibeTokenType::Hardware));
            } else if bridge_kw.contains(&w) {
                tokens.push(VibeToken::new(w, VibeTokenType::Bridge));
            } else if skill_kw.contains(&w) {
                tokens.push(VibeToken::new(w, VibeTokenType::Skill));
            } else if tradition_kw.contains(&w) {
                tokens.push(VibeToken::new(w, VibeTokenType::Tradition));
            } else if action_kw.contains(&w) {
                tokens.push(VibeToken::new(w, VibeTokenType::Action));
            } else if quantity_kw.contains(&w) {
                tokens.push(VibeToken::new(w, VibeTokenType::Quantity));
            } else if modifier_kw.contains(&w) {
                tokens.push(VibeToken::new(w, VibeTokenType::Modifier));
            } else if emotion_kw.contains(&w) {
                tokens.push(VibeToken::new(w, VibeTokenType::Emotion));
            } else {
                // Number quantities
                if let Ok(n) = w.parse::<u32>() {
                    if n > 0 {
                        tokens.push(VibeToken::new(w, VibeTokenType::Quantity));
                    }
                } else {
                    tokens.push(VibeToken::new(w, VibeTokenType::Raw));
                }
            }
        }
        tokens
    }
}

// ---------------------------------------------------------------------------
// 3. VibeAST
// ---------------------------------------------------------------------------

/// Abstract syntax tree of a vibe command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreateSpec {
    pub item_type: ItemType,
    pub name: Option<String>,
    pub quantity: u32,
    pub properties: HashMap<String, String>,
}

impl Default for CreateSpec {
    fn default() -> Self {
        Self {
            item_type: ItemType::Custom,
            name: None,
            quantity: 1,
            properties: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Room,
    Agent,
    Hardware,
    Bridge,
    Skill,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModifySpec {
    pub property: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QuerySpec {
    pub target: String,
    pub question: String,
}

// ---------------------------------------------------------------------------
// 4. VibeParser
// ---------------------------------------------------------------------------

/// Parses `VibeToken` streams into a `VibeAST`.
pub struct VibeParser;

impl VibeParser {
    /// Parse tokens into an AST.
    pub fn parse(tokens: Vec<VibeToken>) -> Result<VibeAST, String> {
        if tokens.is_empty() {
            return Err("empty input".into());
        }

        let types: Vec<VibeTokenType> = tokens.iter().map(|t| t.token_type).collect();

        // Deploy
        if types.contains(&VibeTokenType::Action)
            && tokens.iter().any(|t| t.text == "deploy")
        {
            return Ok(VibeAST::Deploy);
        }

        // Reset
        if types.contains(&VibeTokenType::Action)
            && tokens.iter().any(|t| t.text == "reset")
        {
            return Ok(VibeAST::Reset);
        }

        // LoadProgram: "load X program"
        if let Some(load_idx) = tokens.iter().position(|t| t.text == "load") {
            let raw_after: Vec<&VibeToken> = tokens[load_idx + 1..]
                .iter()
                .filter(|t| t.token_type == VibeTokenType::Raw)
                .collect();
            let name = if raw_after.is_empty() {
                // Check for known program names that are keywords
                tokens[load_idx + 1..]
                    .iter()
                    .find(|t| t.token_type != VibeTokenType::Action)
                    .map(|t| t.text.clone())
                    .unwrap_or_else(|| "unknown".into())
            } else {
                raw_after[0].text.clone()
            };
            return Ok(VibeAST::LoadProgram(name));
        }

        // Test: "test the X"
        if let Some(test_idx) = tokens.iter().position(|t| t.text == "test") {
            let target = Self::find_name(&tokens[test_idx + 1..]);
            return Ok(VibeAST::Test(target));
        }

        // Destroy: "remove X" / "destroy X" / "delete X"
        if let Some(rm_idx) = tokens
            .iter()
            .position(|t| t.text == "remove" || t.text == "destroy" || t.text == "delete")
        {
            let target = Self::find_name(&tokens[rm_idx + 1..]);
            return Ok(VibeAST::Destroy(target));
        }

        // Query: "how is X" / "how X"
        if let Some(how_idx) = tokens.iter().position(|t| t.text == "how") {
            let target = Self::find_name(&tokens[how_idx + 1..]);
            return Ok(VibeAST::Query(QuerySpec {
                target,
                question: "status".into(),
            }));
        }

        // Modify: "make the X Y" / "set X to Y" / "change X Y"
        if let Some(mod_idx) = tokens.iter().position(|t| {
            t.text == "make" || t.text == "set" || t.text == "change"
        }) {
            let remainder = &tokens[mod_idx + 1..];
            // Skip "the"
            let remainder: Vec<&VibeToken> =
                remainder.iter().filter(|t| t.text != "the").collect();
            if remainder.len() >= 2 {
                let target = remainder[0].text.clone();
                let value = remainder[1].text.clone();
                let property = remainder
                    .get(2)
                    .map(|t| t.text.clone())
                    .unwrap_or_else(|| "attribute".into());
                return Ok(VibeAST::Modify(
                    target,
                    ModifySpec {
                        property,
                        value,
                    },
                ));
            } else if !remainder.is_empty() {
                let target = remainder[0].text.clone();
                return Ok(VibeAST::Modify(
                    target,
                    ModifySpec {
                        property: "attribute".into(),
                        value: "modified".into(),
                    },
                ));
            }
        }

        // Create: "I need X" / "create X" / "build X" / "give me X"
        let action_words = ["need", "want", "create", "build", "make", "give"];
        if let Some(act_idx) = tokens
            .iter()
            .position(|t| action_words.contains(&t.text.as_str()))
        {
            let remainder = &tokens[act_idx + 1..];
            return Ok(VibeAST::Create(Self::build_create_spec(remainder)));
        }

        // Fallback: look for any item type token and create from that
        if let Some(item_type) = Self::extract_item_type(&tokens) {
            let quantity = Self::extract_quantity(&tokens);
            let name = Self::find_raw_name(&tokens);
            let properties = Self::extract_properties(&tokens);
            return Ok(VibeAST::Create(CreateSpec {
                item_type,
                name,
                quantity,
                properties,
            }));
        }

        // Last resort: try to extract raw tokens as a freeform create
        let raws: Vec<&VibeToken> =
            tokens.iter().filter(|t| t.token_type == VibeTokenType::Raw).collect();
        if !raws.is_empty() {
            let name = raws[0].text.clone();
            Ok(VibeAST::Create(CreateSpec {
                item_type: ItemType::Custom,
                name: Some(name),
                quantity: 1,
                properties: HashMap::new(),
            }))
        } else {
            Err("could not parse vibe input".into())
        }
    }

    fn build_create_spec(tokens: &[VibeToken]) -> CreateSpec {
        let item_type = Self::extract_item_type(tokens).unwrap_or(ItemType::Custom);
        let quantity = Self::extract_quantity(tokens);
        let name = Self::find_raw_name(tokens);
        let properties = Self::extract_properties(tokens);
        CreateSpec {
            item_type,
            name,
            quantity,
            properties,
        }
    }

    fn extract_item_type(tokens: &[VibeToken]) -> Option<ItemType> {
        for t in tokens {
            match t.token_type {
                VibeTokenType::Room => return Some(ItemType::Room),
                VibeTokenType::Agent => return Some(ItemType::Agent),
                VibeTokenType::Hardware => return Some(ItemType::Hardware),
                VibeTokenType::Bridge => return Some(ItemType::Bridge),
                VibeTokenType::Skill => return Some(ItemType::Skill),
                _ => {}
            }
        }
        None
    }

    fn extract_quantity(tokens: &[VibeToken]) -> u32 {
        use VibeTokenType::Quantity;
        for t in tokens {
            if t.token_type == Quantity {
                if let Ok(n) = t.text.parse::<u32>() {
                    return n;
                }
                // Text quantities
                return match t.text.as_str() {
                    "three" => 3,
                    "four" => 4,
                    "five" => 5,
                    "six" => 6,
                    "seven" => 7,
                    "eight" => 8,
                    "nine" => 9,
                    "ten" => 10,
                    "lots" | "many" | "several" | "all" | "some" => 10,
                    _ => 1,
                };
            }
        }
        1
    }

    fn find_name(tokens: &[VibeToken]) -> String {
        let skip_words = ["the", "a", "an", "is", "are", "does", "was", "were"];
        // Prefer typed tokens (Room, Agent, etc.) over Raw
        for t in tokens {
            if !skip_words.contains(&t.text.as_str()) && t.token_type != VibeTokenType::Raw {
                return t.text.clone();
            }
        }
        // Fall back to any non-skip token
        for t in tokens {
            if !skip_words.contains(&t.text.as_str()) {
                return t.text.clone();
            }
        }
        "unknown".into()
    }

    fn find_raw_name(tokens: &[VibeToken]) -> Option<String> {
        tokens
            .iter()
            .find(|t| t.token_type == VibeTokenType::Raw)
            .map(|t| t.text.clone())
    }

    fn extract_properties(tokens: &[VibeToken]) -> HashMap<String, String> {
        let mut props = HashMap::new();
        for t in tokens {
            match t.token_type {
                VibeTokenType::Tradition => {
                    props.insert("tradition".into(), t.text.clone());
                }
                VibeTokenType::Modifier => {
                    props.insert("modifier".into(), t.text.clone());
                }
                VibeTokenType::Emotion => {
                    props.insert("emotion".into(), t.text.clone());
                }
                _ => {}
            }
        }
        // Also check for tokens that serve as property values
        // e.g., "symmetry" is a skill-related raw, "fractal" is a pattern
        let skill_adj = ["symmetry", "fractal", "combat", "engineering", "exploration", "social"];
        for t in tokens {
            if t.token_type == VibeTokenType::Raw && skill_adj.contains(&t.text.as_str()) {
                props.entry("curriculum".into()).or_insert_with(|| t.text.clone());
            }
        }
        props
    }
}

// ---------------------------------------------------------------------------
// 5. PlatoOp
// ---------------------------------------------------------------------------

/// A PLATO operation — the compiled output of a vibe command.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PlatoOp {
    CreateRoom {
        name: String,
        room_type: String,
        properties: HashMap<String, String>,
    },
    CreateAgent {
        name: String,
        archetype: String,
        skills: Vec<String>,
    },
    CreateHardware {
        name: String,
        hw_type: String,
        channels: u32,
    },
    CreateBridge {
        target: String,
    },
    TrainAgent {
        agent: String,
        skill: String,
        tradition: Option<String>,
    },
    ModifyItem {
        id: String,
        property: String,
        value: String,
    },
    DestroyItem {
        id: String,
    },
    DeployAll,
    ResetAll,
    QueryStatus {
        target: String,
    },
    TestItem {
        id: String,
    },
    LoadProgram {
        name: String,
    },
}

// ---------------------------------------------------------------------------
// 6. VibeCompiler
// ---------------------------------------------------------------------------

/// THE compiler. Vibe → AST → PLATO ops.
pub struct VibeCompiler {
    counter: std::cell::Cell<u64>,
}

impl VibeCompiler {
    pub fn new() -> Self {
        Self {
            counter: std::cell::Cell::new(0),
        }
    }

    fn next_id(&self) -> u64 {
        let n = self.counter.get() + 1;
        self.counter.set(n);
        n
    }

    /// Full pipeline: lex → parse → compile.
    pub fn compile(&self, input: &str) -> Result<Vec<PlatoOp>, String> {
        let tokens = VibeLexer::lex(input);
        let ast = VibeParser::parse(tokens)?;
        Ok(self.compile_ast(ast))
    }

    /// Tokenize only.
    pub fn lex(&self, input: &str) -> Vec<VibeToken> {
        VibeLexer::lex(input)
    }

    /// Parse tokens into AST.
    pub fn parse(&self, tokens: Vec<VibeToken>) -> Result<VibeAST, String> {
        VibeParser::parse(tokens)
    }

    /// Compile a single AST node into PLATO ops.
    pub fn compile_ast(&self, ast: VibeAST) -> Vec<PlatoOp> {
        match ast {
            VibeAST::Create(spec) => self.compile_create(spec),
            VibeAST::Modify(target, mod_spec) => {
                vec![PlatoOp::ModifyItem {
                    id: target,
                    property: mod_spec.property,
                    value: mod_spec.value,
                }]
            }
            VibeAST::Destroy(target) => {
                vec![PlatoOp::DestroyItem { id: target }]
            }
            VibeAST::Query(q) => {
                vec![PlatoOp::QueryStatus { target: q.target }]
            }
            VibeAST::Deploy => vec![PlatoOp::DeployAll],
            VibeAST::Reset => vec![PlatoOp::ResetAll],
            VibeAST::LoadProgram(name) => {
                vec![PlatoOp::LoadProgram { name }]
            }
            VibeAST::Test(id) => {
                vec![PlatoOp::TestItem { id }]
            }
        }
    }

    fn compile_create(&self, spec: CreateSpec) -> Vec<PlatoOp> {
        let count = spec.quantity.max(1);
        let mut ops = Vec::new();

        for i in 0..count {
            let id = self.next_id();
            let name = spec
                .name
                .clone()
                .unwrap_or_else(|| format!("{}_{}", spec.item_type.item_type_label(), id));

            let display_name = if count > 1 {
                format!("{}_{}", name, i + 1)
            } else {
                name
            };

            match spec.item_type {
                ItemType::Room => {
                    ops.push(PlatoOp::CreateRoom {
                        name: display_name,
                        room_type: spec
                            .properties
                            .get("curriculum")
                            .cloned()
                            .unwrap_or_else(|| "general".into()),
                        properties: spec.properties.clone(),
                    });
                }
                ItemType::Agent => {
                    let tradition = spec.properties.get("tradition").cloned();
                    let skill = spec
                        .properties
                        .get("curriculum")
                        .cloned()
                        .unwrap_or_else(|| "general".into());
                    ops.push(PlatoOp::CreateAgent {
                        name: display_name.clone(),
                        archetype: spec
                            .properties
                            .get("modifier")
                            .cloned()
                            .unwrap_or_else(|| "default".into()),
                        skills: vec![skill.clone()],
                    });
                    if let Some(t) = tradition {
                        ops.push(PlatoOp::TrainAgent {
                            agent: display_name.clone(),
                            skill,
                            tradition: Some(t),
                        });
                    }
                }
                ItemType::Hardware => {
                    ops.push(PlatoOp::CreateHardware {
                        name: display_name,
                        hw_type: spec
                            .properties
                            .get("modifier")
                            .cloned()
                            .unwrap_or_else(|| "generic".into()),
                        channels: 1,
                    });
                }
                ItemType::Bridge => {
                    let target = spec
                        .name
                        .clone()
                        .unwrap_or_else(|| "unknown".into());
                    ops.push(PlatoOp::CreateBridge { target });
                }
                ItemType::Skill => {
                    let skill_name = spec
                        .properties
                        .get("curriculum")
                        .cloned()
                        .unwrap_or_else(|| "general".into());
                    let tradition = spec.properties.get("tradition").cloned();
                    ops.push(PlatoOp::TrainAgent {
                        agent: display_name,
                        skill: skill_name,
                        tradition,
                    });
                }
                ItemType::Custom => {
                    ops.push(PlatoOp::CreateRoom {
                        name: display_name,
                        room_type: "custom".into(),
                        properties: spec.properties.clone(),
                    });
                }
            }
        }
        ops
    }

    /// Compile multiple commands at once.
    pub fn batch_compile(&self, inputs: Vec<&str>) -> Vec<Result<Vec<PlatoOp>, String>> {
        inputs.into_iter().map(|i| self.compile(i)).collect()
    }
}

impl Default for VibeCompiler {
    fn default() -> Self {
        Self::new()
    }
}

trait ItemTypeLabel {
    fn item_type_label(&self) -> &'static str;
}

impl ItemTypeLabel for ItemType {
    fn item_type_label(&self) -> &'static str {
        match self {
            ItemType::Room => "room",
            ItemType::Agent => "agent",
            ItemType::Hardware => "hw",
            ItemType::Bridge => "bridge",
            ItemType::Skill => "skill",
            ItemType::Custom => "item",
        }
    }
}

// ---------------------------------------------------------------------------
// 7. VibeProgram
// ---------------------------------------------------------------------------

/// A saved program — like Neo loading combat training.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VibeProgram {
    pub name: String,
    pub description: String,
    pub ops: Vec<PlatoOp>,
}

/// Result of executing a program.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProgramResult {
    pub ops_executed: usize,
    pub success: bool,
    pub message: String,
}

impl VibeProgram {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            ops: Vec::new(),
        }
    }

    pub fn add_op(&mut self, op: PlatoOp) {
        self.ops.push(op);
    }

    pub fn execute(&self) -> ProgramResult {
        let n = self.ops.len();
        ProgramResult {
            ops_executed: n,
            success: n > 0,
            message: format!("Executed {} ops from program '{}'", n, self.name),
        }
    }
}

// ---------------------------------------------------------------------------
// 8. Pre-built programs
// ---------------------------------------------------------------------------

/// Training rooms + combat agents + skill loading.
pub fn combat_program() -> VibeProgram {
    let mut p = VibeProgram::new("combat");
    p.description = "Combat training: rooms, agents, and skill loading".into();
    p.add_op(PlatoOp::CreateRoom {
        name: "training_hall".into(),
        room_type: "Training".into(),
        properties: {
            let mut h = HashMap::new();
            h.insert("curriculum".into(), "Combat".into());
            h
        },
    });
    p.add_op(PlatoOp::CreateAgent {
        name: "warrior".into(),
        archetype: "fighter".into(),
        skills: vec!["combat".into(), "strategy".into()],
    });
    p.add_op(PlatoOp::TrainAgent {
        agent: "warrior".into(),
        skill: "combat".into(),
        tradition: Some("greek".into()),
    });
    p
}

/// Hardware rooms + motor agents + sensor arrays.
pub fn engineering_program() -> VibeProgram {
    let mut p = VibeProgram::new("engineering");
    p.description = "Engineering: hardware rooms, motor agents, sensor arrays".into();
    p.add_op(PlatoOp::CreateRoom {
        name: "workshop".into(),
        room_type: "Engineering".into(),
        properties: HashMap::new(),
    });
    p.add_op(PlatoOp::CreateHardware {
        name: "motor_array".into(),
        hw_type: "servo".into(),
        channels: 8,
    });
    p.add_op(PlatoOp::CreateAgent {
        name: "mechanic".into(),
        archetype: "engineer".into(),
        skills: vec!["motor_control".into()],
    });
    p
}

/// Scout agents + mapping rooms + terrain generators.
pub fn exploration_program() -> VibeProgram {
    let mut p = VibeProgram::new("exploration");
    p.description = "Exploration: scout agents, mapping rooms, terrain generators".into();
    p.add_op(PlatoOp::CreateAgent {
        name: "scout".into(),
        archetype: "explorer".into(),
        skills: vec!["mapping".into(), "terrain".into()],
    });
    p.add_op(PlatoOp::CreateRoom {
        name: "map_room".into(),
        room_type: "Mapping".into(),
        properties: HashMap::new(),
    });
    p
}

/// Palaver rooms + diplomacy agents + bridges.
pub fn social_program() -> VibeProgram {
    let mut p = VibeProgram::new("social");
    p.description = "Social: palaver rooms, diplomacy agents, bridges".into();
    p.add_op(PlatoOp::CreateRoom {
        name: "palaver".into(),
        room_type: "Diplomacy".into(),
        properties: {
            let mut h = HashMap::new();
            h.insert("tradition".into(), "african".into());
            h
        },
    });
    p.add_op(PlatoOp::CreateAgent {
        name: "diplomat".into(),
        archetype: "mediator".into(),
        skills: vec!["diplomacy".into()],
    });
    p.add_op(PlatoOp::CreateBridge {
        target: "all_factions".into(),
    });
    p
}

// ---------------------------------------------------------------------------
// 9. VoiceAnnotation
// ---------------------------------------------------------------------------

/// Extra data from voice input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceAnnotation {
    pub transcription: String,
    pub confidence: f64,
    pub language: String,
    pub prosody: Option<Prosody>,
}

/// Prosody — emotional context extracted from voice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prosody {
    pub energy: f64,
    pub pace: f64,
    pub pitch: f64,
}

impl VoiceAnnotation {
    /// Create a voice annotation from plain text (no voice data).
    pub fn from_text(text: &str) -> Self {
        Self {
            transcription: text.into(),
            confidence: 1.0,
            language: "en".into(),
            prosody: None,
        }
    }
}

// ===========================================================================
// TESTS
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- VibeToken ---
    #[test]
    fn test_vibe_token_new() {
        let t = VibeToken::new("room", VibeTokenType::Room);
        assert_eq!(t.text, "room");
        assert_eq!(t.token_type, VibeTokenType::Room);
        assert_eq!(t.confidence, 1.0);
    }

    #[test]
    fn test_vibe_token_serialization() {
        let t = VibeToken::new("agent", VibeTokenType::Agent);
        let json = serde_json::to_string(&t).unwrap();
        assert!(json.contains("\"text\":\"agent\""));
        let back: VibeToken = serde_json::from_str(&json).unwrap();
        assert_eq!(back, t);
    }

    // --- VibeLexer ---
    #[test]
    fn test_lex_room() {
        let tokens = VibeLexer::lex("I need a room");
        assert!(tokens.iter().any(|t| t.token_type == VibeTokenType::Room));
    }

    #[test]
    fn test_lex_agent() {
        let tokens = VibeLexer::lex("create an agent");
        assert!(tokens.iter().any(|t| t.token_type == VibeTokenType::Agent));
    }

    #[test]
    fn test_lex_hardware() {
        let tokens = VibeLexer::lex("connect a servo");
        assert!(tokens.iter().any(|t| t.token_type == VibeTokenType::Hardware));
    }

    #[test]
    fn test_lex_bridge() {
        let tokens = VibeLexer::lex("build a bridge");
        assert!(tokens.iter().any(|t| t.token_type == VibeTokenType::Bridge));
    }

    #[test]
    fn test_lex_tradition() {
        let tokens = VibeLexer::lex("learn african patterns");
        assert!(tokens.iter().any(|t| t.token_type == VibeTokenType::Tradition && t.text == "african"));
    }

    #[test]
    fn test_lex_quantity_numeric() {
        let tokens = VibeLexer::lex("I need 3 rooms");
        assert!(tokens.iter().any(|t| t.token_type == VibeTokenType::Quantity && t.text == "3"));
    }

    #[test]
    fn test_lex_quantity_word() {
        let tokens = VibeLexer::lex("I need three agents");
        assert!(tokens.iter().any(|t| t.token_type == VibeTokenType::Quantity && t.text == "three"));
    }

    #[test]
    fn test_lex_modifier() {
        let tokens = VibeLexer::lex("build a big room");
        assert!(tokens.iter().any(|t| t.token_type == VibeTokenType::Modifier && t.text == "big"));
    }

    #[test]
    fn test_lex_action() {
        let tokens = VibeLexer::lex("create a space");
        assert!(tokens.iter().any(|t| t.token_type == VibeTokenType::Action && t.text == "create"));
    }

    #[test]
    fn test_lex_multiple_traditions() {
        let tokens = VibeLexer::lex("greek and japanese traditions");
        let traditions: Vec<&VibeToken> = tokens.iter().filter(|t| t.token_type == VibeTokenType::Tradition).collect();
        assert_eq!(traditions.len(), 2);
    }

    #[test]
    fn test_lex_empty() {
        let tokens = VibeLexer::lex("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_lex_emotion() {
        let tokens = VibeLexer::lex("a calm agent");
        assert!(tokens.iter().any(|t| t.token_type == VibeTokenType::Emotion && t.text == "calm"));
    }

    #[test]
    fn test_lex_skill_keywords() {
        let tokens = VibeLexer::lex("train the agent");
        assert!(tokens.iter().any(|t| t.token_type == VibeTokenType::Skill));
    }

    // --- VibeParser ---
    #[test]
    fn test_parse_create_room() {
        let tokens = VibeLexer::lex("I need a room");
        let ast = VibeParser::parse(tokens).unwrap();
        match ast {
            VibeAST::Create(spec) => assert_eq!(spec.item_type, ItemType::Room),
            other => panic!("expected Create, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_create_with_quantity() {
        let tokens = VibeLexer::lex("I need 3 rooms");
        let ast = VibeParser::parse(tokens).unwrap();
        match ast {
            VibeAST::Create(spec) => {
                assert_eq!(spec.item_type, ItemType::Room);
                assert_eq!(spec.quantity, 3);
            }
            other => panic!("expected Create, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_deploy() {
        let tokens = VibeLexer::lex("deploy everything");
        let ast = VibeParser::parse(tokens).unwrap();
        assert_eq!(ast, VibeAST::Deploy);
    }

    #[test]
    fn test_parse_reset() {
        let tokens = VibeLexer::lex("reset all");
        let ast = VibeParser::parse(tokens).unwrap();
        assert_eq!(ast, VibeAST::Reset);
    }

    #[test]
    fn test_parse_destroy() {
        let tokens = VibeLexer::lex("remove the room");
        let ast = VibeParser::parse(tokens).unwrap();
        match ast {
            VibeAST::Destroy(id) => assert_eq!(id, "room"),
            other => panic!("expected Destroy, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_query() {
        let tokens = VibeLexer::lex("how is the agent");
        let ast = VibeParser::parse(tokens).unwrap();
        match ast {
            VibeAST::Query(q) => assert_eq!(q.target, "agent"),
            other => panic!("expected Query, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_modify() {
        let tokens = VibeLexer::lex("make the room big");
        let ast = VibeParser::parse(tokens).unwrap();
        match ast {
            VibeAST::Modify(target, spec) => {
                assert_eq!(target, "room");
                assert_eq!(spec.value, "big");
            }
            other => panic!("expected Modify, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_load_program() {
        let tokens = VibeLexer::lex("load combat program");
        let ast = VibeParser::parse(tokens).unwrap();
        match ast {
            VibeAST::LoadProgram(name) => assert_eq!(name, "combat"),
            other => panic!("expected LoadProgram, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_test() {
        let tokens = VibeLexer::lex("test the room");
        let ast = VibeParser::parse(tokens).unwrap();
        match ast {
            VibeAST::Test(id) => assert_eq!(id, "room"),
            other => panic!("expected Test, got {:?}", other),
        }
    }

    #[test]
    fn test_parse_empty_error() {
        let result = VibeParser::parse(vec![]);
        assert!(result.is_err());
    }

    // --- VibeCompiler ---
    #[test]
    fn test_compile_create_room() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("I need a room").unwrap();
        assert!(ops.len() >= 1);
        assert!(matches!(&ops[0], PlatoOp::CreateRoom { .. }));
    }

    #[test]
    fn test_compile_create_agent_with_tradition() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("I need an agent to learn african symmetry").unwrap();
        assert!(ops.iter().any(|op| matches!(op, PlatoOp::CreateAgent { .. })));
        assert!(ops.iter().any(|op| matches!(op, PlatoOp::TrainAgent { .. })));
    }

    #[test]
    fn test_compile_deploy() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("deploy").unwrap();
        assert_eq!(ops, vec![PlatoOp::DeployAll]);
    }

    #[test]
    fn test_compile_reset() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("reset").unwrap();
        assert_eq!(ops, vec![PlatoOp::ResetAll]);
    }

    #[test]
    fn test_compile_destroy() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("remove the room").unwrap();
        assert_eq!(ops, vec![PlatoOp::DestroyItem { id: "room".into() }]);
    }

    #[test]
    fn test_compile_query() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("how is the agent").unwrap();
        assert!(matches!(&ops[0], PlatoOp::QueryStatus { target } if target == "agent"));
    }

    #[test]
    fn test_compile_modify() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("make the room big").unwrap();
        assert!(matches!(&ops[0], PlatoOp::ModifyItem { id, property, value }
            if id == "room" && value == "big"));
    }

    #[test]
    fn test_compile_test() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("test the agent").unwrap();
        assert!(matches!(&ops[0], PlatoOp::TestItem { id } if id == "agent"));
    }

    #[test]
    fn test_compile_load_program() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("load combat program").unwrap();
        assert!(matches!(&ops[0], PlatoOp::LoadProgram { name } if name == "combat"));
    }

    #[test]
    fn test_batch_compile() {
        let compiler = VibeCompiler::new();
        let results = compiler.batch_compile(vec!["deploy", "reset"]);
        assert_eq!(results.len(), 2);
        assert!(results[0].is_ok());
        assert!(results[1].is_ok());
    }

    #[test]
    fn test_compile_multiple_rooms() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("I need 3 rooms").unwrap();
        let rooms: Vec<_> = ops.iter().filter(|op| matches!(op, PlatoOp::CreateRoom { .. })).collect();
        assert_eq!(rooms.len(), 3);
    }

    #[test]
    fn test_compile_create_bridge() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("build a bridge").unwrap();
        assert!(ops.iter().any(|op| matches!(op, PlatoOp::CreateBridge { .. })));
    }

    #[test]
    fn test_compile_hardware() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("create a servo controller").unwrap();
        assert!(ops.iter().any(|op| matches!(op, PlatoOp::CreateHardware { .. })));
    }

    // --- VibeProgram ---
    #[test]
    fn test_program_new() {
        let p = VibeProgram::new("test");
        assert_eq!(p.name, "test");
        assert!(p.ops.is_empty());
    }

    #[test]
    fn test_program_add_op() {
        let mut p = VibeProgram::new("test");
        p.add_op(PlatoOp::DeployAll);
        assert_eq!(p.ops.len(), 1);
    }

    #[test]
    fn test_program_execute() {
        let mut p = VibeProgram::new("test");
        p.add_op(PlatoOp::DeployAll);
        p.add_op(PlatoOp::ResetAll);
        let result = p.execute();
        assert_eq!(result.ops_executed, 2);
        assert!(result.success);
    }

    #[test]
    fn test_program_execute_empty() {
        let p = VibeProgram::new("empty");
        let result = p.execute();
        assert_eq!(result.ops_executed, 0);
        assert!(!result.success);
    }

    #[test]
    fn test_program_serialization() {
        let p = combat_program();
        let json = serde_json::to_string(&p).unwrap();
        let back: VibeProgram = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, p.name);
        assert_eq!(back.ops.len(), p.ops.len());
    }

    // --- Pre-built programs ---
    #[test]
    fn test_combat_program() {
        let p = combat_program();
        assert_eq!(p.name, "combat");
        assert!(p.ops.len() >= 3);
        let result = p.execute();
        assert!(result.success);
    }

    #[test]
    fn test_engineering_program() {
        let p = engineering_program();
        assert_eq!(p.name, "engineering");
        assert!(p.ops.len() >= 2);
        let result = p.execute();
        assert!(result.success);
    }

    #[test]
    fn test_exploration_program() {
        let p = exploration_program();
        assert_eq!(p.name, "exploration");
        assert!(p.ops.len() >= 2);
        let result = p.execute();
        assert!(result.success);
    }

    #[test]
    fn test_social_program() {
        let p = social_program();
        assert_eq!(p.name, "social");
        assert!(p.ops.len() >= 2);
        let result = p.execute();
        assert!(result.success);
    }

    // --- VoiceAnnotation ---
    #[test]
    fn test_voice_annotation_from_text() {
        let va = VoiceAnnotation::from_text("hello world");
        assert_eq!(va.transcription, "hello world");
        assert_eq!(va.confidence, 1.0);
        assert!(va.prosody.is_none());
    }

    #[test]
    fn test_voice_annotation_serialization() {
        let va = VoiceAnnotation::from_text("test");
        let json = serde_json::to_string(&va).unwrap();
        let back: VoiceAnnotation = serde_json::from_str(&json).unwrap();
        assert_eq!(back.transcription, va.transcription);
    }

    #[test]
    fn test_voice_annotation_with_prosody() {
        let mut va = VoiceAnnotation::from_text("excited speech");
        va.prosody = Some(Prosody {
            energy: 0.9,
            pace: 1.2,
            pitch: 0.7,
        });
        assert!(va.prosody.is_some());
        let p = va.prosody.unwrap();
        assert!((p.energy - 0.9).abs() < f64::EPSILON);
    }

    // --- Integration / end-to-end ---
    #[test]
    fn test_e2e_create_room_with_tradition() {
        let compiler = VibeCompiler::new();
        let ops = compiler
            .compile("I need a room where agents learn symmetry through african fractals")
            .unwrap();
        assert!(!ops.is_empty());
        // Should create a room with curriculum and tradition properties
        let room_op = ops.iter().find(|op| matches!(op, PlatoOp::CreateRoom { .. }));
        assert!(room_op.is_some());
        if let Some(PlatoOp::CreateRoom { properties, .. }) = room_op {
            assert!(properties.contains_key("tradition"));
        }
    }

    #[test]
    fn test_e2e_build_fast_agent() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("build a fast agent").unwrap();
        assert!(ops.iter().any(|op| matches!(op, PlatoOp::CreateAgent { .. })));
    }

    #[test]
    fn test_e2e_connect_bridge() {
        let compiler = VibeCompiler::new();
        let ops = compiler.compile("connect a bridge to the portal").unwrap();
        assert!(ops.iter().any(|op| matches!(op, PlatoOp::CreateBridge { .. })));
    }

    #[test]
    fn test_plato_op_serialization() {
        let ops = vec![
            PlatoOp::DeployAll,
            PlatoOp::CreateRoom {
                name: "test".into(),
                room_type: "general".into(),
                properties: HashMap::new(),
            },
        ];
        let json = serde_json::to_string(&ops).unwrap();
        let back: Vec<PlatoOp> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ops);
    }

    #[test]
    fn test_compiler_default() {
        let compiler = VibeCompiler::default();
        let ops = compiler.compile("deploy").unwrap();
        assert_eq!(ops, vec![PlatoOp::DeployAll]);
    }
}
