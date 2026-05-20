//! The aelm-mcp server: tool router + `ServerHandler` (resources).

use rmcp::handler::server::router::prompt::PromptRouter;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, GetPromptRequestParams, GetPromptResult, Implementation, ListPromptsResult,
    ListResourcesResult, PaginatedRequestParams, PromptMessage, ProtocolVersion,
    ReadResourceRequestParams, ReadResourceResult, ServerCapabilities, ServerInfo,
};
use rmcp::service::RequestContext;
use rmcp::{
    prompt, prompt_handler, prompt_router, tool, tool_handler, tool_router, ErrorData, RoleServer,
    ServerHandler,
};
use serde::Deserialize;

use crate::cli_runner::AelmCli;
use crate::tools::generate::ScaffoldPart;
use crate::{prompts, resources, tools};

/// Shared server state: the configured CLI runner.
#[derive(Clone)]
pub struct AelmMcpServer {
    cli: AelmCli,
    tool_router: ToolRouter<Self>,
    prompt_router: PromptRouter<Self>,
}

// ── Tool parameter structs ──────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SourceArgs {
    /// Aelm circuit source (`.aelm` text).
    pub source: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RenderArgs {
    /// Aelm circuit source (`.aelm` text).
    pub source: String,
    /// Theme: `light` (default) or `dark`.
    #[serde(default)]
    pub theme: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RenderPngArgs {
    /// Aelm circuit source (`.aelm` text).
    pub source: String,
    /// PNG resolution in DPI (default 150).
    #[serde(default)]
    pub dpi: Option<u32>,
    /// Theme: `light` (default) or `dark`.
    #[serde(default)]
    pub theme: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct NameArgs {
    /// The part / symbol / example name.
    pub name: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CategoryArgs {
    /// Optional category filter.
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SearchArgs {
    /// Search query.
    pub query: String,
    /// Maximum number of results (default 20).
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct PartInfoArgs {
    /// Part name.
    pub name: String,
    /// Include an inline SVG rendering of the part's symbol.
    #[serde(default)]
    pub render_symbol: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExampleGetArgs {
    /// Example name.
    pub name: String,
    /// Include a rendered SVG of the example.
    #[serde(default)]
    pub render: bool,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DocArgs {
    /// Topic key: `dsl`, `cli`, `style`, `patterns`, `drc`, `mistakes`.
    #[serde(default)]
    pub topic: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AnalyzeArgs {
    /// Aelm circuit source (`.aelm` text).
    pub source: String,
    /// Restrict analysis to a single module by name.
    #[serde(default)]
    pub module: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MoveArgs {
    /// Aelm circuit source (`.aelm` text).
    pub source: String,
    /// Instance name (e.g. `R1`).
    pub instance: String,
    /// Target X in mm (grid-snapped by the CLI).
    pub x: f64,
    /// Target Y in mm (grid-snapped by the CLI).
    pub y: f64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RotateArgs {
    /// Aelm circuit source (`.aelm` text).
    pub source: String,
    /// Instance name.
    pub instance: String,
    /// Rotation in degrees.
    pub degrees: i64,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MirrorArgs {
    /// Aelm circuit source (`.aelm` text).
    pub source: String,
    /// Instance name.
    pub instance: String,
    /// Mirror axis: `x` or `y`.
    pub axis: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ConnectionArgs {
    /// Aelm circuit source (`.aelm` text).
    pub source: String,
    /// Source pin (e.g. `R1.p2`).
    pub from_pin: String,
    /// Destination pin (e.g. `C1.p1`).
    pub to_pin: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScaffoldArgs {
    /// Module name for the generated skeleton.
    pub module: String,
    /// Instances to scaffold, as `{ name, type_name }` objects.
    pub parts: Vec<ScaffoldPartArg>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ScaffoldPartArg {
    /// Instance name (e.g. `R1`).
    pub name: String,
    /// Part type (e.g. `Resistor`).
    pub type_name: String,
}

// ── Prompt parameter structs ────────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DesignArgs {
    /// What to design (requirements in natural language).
    pub description: String,
    /// Optional constraints (budget, size, voltage, …).
    #[serde(default)]
    pub constraints: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SourceArg {
    /// Aelm circuit source (`.aelm` text).
    pub source: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DebugArgs {
    /// Aelm circuit source (`.aelm` text).
    pub source: String,
    /// Optional comma-separated DRC codes to focus on.
    #[serde(default)]
    pub error_codes: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct RequirementsArg {
    /// Design requirements for part selection.
    pub requirements: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LevelArg {
    /// Tutorial level: `beginner`, `intermediate`, or `advanced`.
    pub level: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GoalArg {
    /// The design goal to iterate toward.
    pub goal: String,
}

// ── Phase 3 parameter structs ───────────────────────────────────────────────

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ExtractArgs {
    /// Aelm circuit source (`.aelm` text).
    pub source: String,
    /// Comma-separated instance names to extract (e.g. `R1,R2,C1`).
    pub instances: String,
    /// Restrict to a single module by name.
    #[serde(default)]
    pub module: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct LibraryPathArg {
    /// Path to a `.alib` file or a directory of them.
    pub path: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SvgArg {
    /// SVG artwork to compile into a symbol block.
    pub svg: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct DiffArgs {
    /// The "before" circuit source.
    pub before: String,
    /// The "after" circuit source.
    pub after: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct BatchArgs {
    /// Circuit sources to render concurrently.
    pub sources: Vec<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct QueryArg {
    /// Search query (e.g. `voltage divider`, `low-pass filter`).
    pub query: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct SimilarArgs {
    /// The reference circuit source.
    pub source: String,
    /// Candidate circuit sources to compare against.
    pub candidates: Vec<String>,
}

// ── Tool router ─────────────────────────────────────────────────────────────

#[tool_router]
impl AelmMcpServer {
    pub fn new(cli: AelmCli) -> Self {
        Self {
            cli,
            tool_router: Self::tool_router(),
            prompt_router: Self::prompt_router(),
        }
    }

    #[tool(
        description = "Parse Aelm circuit source into structured IR (modules, parts, connections)."
    )]
    async fn parse(&self, Parameters(a): Parameters<SourceArgs>) -> CallToolResult {
        tools::parse::parse(&self.cli, &a.source).await
    }

    #[tool(
        description = "Validate a circuit: parse plus DRC checks. Returns diagnostics with severity and source spans."
    )]
    async fn validate(&self, Parameters(a): Parameters<SourceArgs>) -> CallToolResult {
        tools::parse::validate(&self.cli, &a.source).await
    }

    #[tool(description = "Format Aelm source code and return the formatted text.")]
    async fn format(&self, Parameters(a): Parameters<SourceArgs>) -> CallToolResult {
        tools::parse::format(&self.cli, &a.source).await
    }

    #[tool(
        description = "Render a circuit to an SVG image (returns SVG text and an inline image)."
    )]
    async fn render_svg(&self, Parameters(a): Parameters<RenderArgs>) -> CallToolResult {
        tools::render::render_svg(&self.cli, &a.source, a.theme.as_deref()).await
    }

    #[tool(description = "Render a circuit to a PNG image (returns an inline image).")]
    async fn render_png(&self, Parameters(a): Parameters<RenderPngArgs>) -> CallToolResult {
        tools::render::render_png(&self.cli, &a.source, a.dpi, a.theme.as_deref()).await
    }

    #[tool(
        description = "Render a single symbol template to SVG by name (e.g. `resistor`, `op_amp`)."
    )]
    async fn render_symbol(&self, Parameters(a): Parameters<NameArgs>) -> CallToolResult {
        tools::render::render_symbol(&self.cli, &a.name).await
    }

    #[tool(description = "List parts from the standard library and any configured user libraries.")]
    async fn list_parts(&self, Parameters(a): Parameters<CategoryArgs>) -> CallToolResult {
        tools::parts::list_parts(&self.cli, a.category.as_deref()).await
    }

    #[tool(description = "Fuzzy-search parts by name, description, or category.")]
    async fn search_parts(&self, Parameters(a): Parameters<SearchArgs>) -> CallToolResult {
        tools::parts::search_parts(&self.cli, &a.query, a.limit).await
    }

    #[tool(
        description = "Get detailed information about a part, optionally with an inline symbol SVG."
    )]
    async fn get_part_info(&self, Parameters(a): Parameters<PartInfoArgs>) -> CallToolResult {
        tools::parts::get_part_info(&self.cli, &a.name, a.render_symbol).await
    }

    #[tool(description = "List built-in and user-defined symbol templates.")]
    async fn list_symbols(&self, Parameters(a): Parameters<CategoryArgs>) -> CallToolResult {
        tools::symbols::list_symbols(&self.cli, a.category.as_deref()).await
    }

    #[tool(
        description = "Get detailed information about a symbol template (pins, bounds, draw commands)."
    )]
    async fn get_symbol_info(&self, Parameters(a): Parameters<NameArgs>) -> CallToolResult {
        tools::symbols::get_symbol_info(&self.cli, &a.name).await
    }

    #[tool(description = "List bundled example circuits.")]
    async fn list_examples(&self, Parameters(a): Parameters<CategoryArgs>) -> CallToolResult {
        tools::examples::list_examples(&self.cli, a.category.as_deref()).await
    }

    #[tool(description = "Get an example circuit's source, optionally with a rendered SVG.")]
    async fn get_example(&self, Parameters(a): Parameters<ExampleGetArgs>) -> CallToolResult {
        tools::examples::get_example(&self.cli, &a.name, a.render).await
    }

    #[tool(
        description = "Get embedded Aelm reference documentation by topic (dsl, cli, style, patterns, drc, mistakes)."
    )]
    async fn get_dsl_reference(&self, Parameters(a): Parameters<DocArgs>) -> CallToolResult {
        tools::docs::get_reference(a.topic.as_deref())
    }

    // ── Analysis (Phase 2) ──────────────────────────────────────────────

    #[tool(
        description = "Analyze a circuit: per-module instance/connection/net counts, parts used, DRC summary, and a complexity estimate."
    )]
    async fn analyze_project(&self, Parameters(a): Parameters<SourceArgs>) -> CallToolResult {
        tools::project::analyze_project(&self.cli, &a.source).await
    }

    #[tool(
        description = "Extract the net connectivity graph: each net's name, electrical type, and member pins."
    )]
    async fn extract_netlist(&self, Parameters(a): Parameters<AnalyzeArgs>) -> CallToolResult {
        tools::project::extract_netlist(&self.cli, &a.source, a.module.as_deref()).await
    }

    #[tool(
        description = "Generate a bill of materials grouped by part and value, with footprints and reference designators."
    )]
    async fn extract_bom(&self, Parameters(a): Parameters<AnalyzeArgs>) -> CallToolResult {
        tools::project::extract_bom(&self.cli, &a.source, a.module.as_deref()).await
    }

    // ── Edit (Phase 2, dry-run) ─────────────────────────────────────────

    #[tool(
        description = "Move an instance to (x, y). Returns the modified source and a line diff; the input is never mutated."
    )]
    async fn apply_move(&self, Parameters(a): Parameters<MoveArgs>) -> CallToolResult {
        tools::edit::apply_move(&self.cli, &a.source, &a.instance, a.x, a.y).await
    }

    #[tool(description = "Rotate an instance by a number of degrees. Returns modified source.")]
    async fn apply_rotate(&self, Parameters(a): Parameters<RotateArgs>) -> CallToolResult {
        tools::edit::apply_rotate(&self.cli, &a.source, &a.instance, a.degrees).await
    }

    #[tool(description = "Mirror an instance across the `x` or `y` axis. Returns modified source.")]
    async fn apply_mirror(&self, Parameters(a): Parameters<MirrorArgs>) -> CallToolResult {
        tools::edit::apply_mirror(&self.cli, &a.source, &a.instance, &a.axis).await
    }

    #[tool(description = "Add a connection between two pins. Returns modified source.")]
    async fn apply_add_connection(
        &self,
        Parameters(a): Parameters<ConnectionArgs>,
    ) -> CallToolResult {
        tools::edit::apply_add_connection(&self.cli, &a.source, &a.from_pin, &a.to_pin).await
    }

    #[tool(description = "Delete a connection between two pins. Returns modified source.")]
    async fn apply_delete_connection(
        &self,
        Parameters(a): Parameters<ConnectionArgs>,
    ) -> CallToolResult {
        tools::edit::apply_delete_connection(&self.cli, &a.source, &a.from_pin, &a.to_pin).await
    }

    // ── Generation (Phase 2) ────────────────────────────────────────────

    #[tool(
        description = "Generate a valid .aelm skeleton from a module name and a list of parts, then validate it."
    )]
    async fn scaffold_circuit(&self, Parameters(a): Parameters<ScaffoldArgs>) -> CallToolResult {
        let parts: Vec<ScaffoldPart> = a
            .parts
            .into_iter()
            .map(|p| ScaffoldPart {
                name: p.name,
                type_name: p.type_name,
            })
            .collect();
        tools::generate::scaffold_circuit(&self.cli, &a.module, &parts).await
    }

    // ── Advanced analysis (Phase 3) ─────────────────────────────────────

    #[tool(description = "Topological analysis: isolated instances and per-net fan-out.")]
    async fn analyze_connectivity(&self, Parameters(a): Parameters<AnalyzeArgs>) -> CallToolResult {
        tools::project::analyze_connectivity(&self.cli, &a.source, a.module.as_deref()).await
    }

    #[tool(
        description = "Extract a set of instances and their mutual connections into a standalone module."
    )]
    async fn extract_subcircuit(&self, Parameters(a): Parameters<ExtractArgs>) -> CallToolResult {
        tools::project::extract_subcircuit(&self.cli, &a.source, &a.instances, a.module.as_deref())
            .await
    }

    #[tool(
        description = "Structural diff between two circuit versions (added/removed/changed instances)."
    )]
    async fn diff_circuits(&self, Parameters(a): Parameters<DiffArgs>) -> CallToolResult {
        tools::advanced::diff_circuits(&self.cli, &a.before, &a.after).await
    }

    #[tool(description = "Render multiple circuits to SVG concurrently.")]
    async fn render_batch(&self, Parameters(a): Parameters<BatchArgs>) -> CallToolResult {
        tools::advanced::render_batch(&self.cli, &a.sources).await
    }

    #[tool(description = "Render a circuit two ways (light vs dark) for visual comparison.")]
    async fn preview_style(&self, Parameters(a): Parameters<RenderArgs>) -> CallToolResult {
        tools::advanced::preview_style(&self.cli, &a.source, a.theme.as_deref()).await
    }

    #[tool(description = "Suggest placement: list instances missing an explicit place: hint.")]
    async fn suggest_placement(&self, Parameters(a): Parameters<SourceArgs>) -> CallToolResult {
        tools::advanced::suggest_placement(&self.cli, &a.source).await
    }

    #[tool(
        description = "Evaluate calc/plot expressions by rendering the circuit and returning the data."
    )]
    async fn evaluate_calc(&self, Parameters(a): Parameters<SourceArgs>) -> CallToolResult {
        tools::advanced::evaluate_calc(&self.cli, &a.source).await
    }

    #[tool(description = "Suggest a circuit pattern from the embedded pattern library by query.")]
    async fn suggest_circuit_pattern(&self, Parameters(a): Parameters<QueryArg>) -> CallToolResult {
        tools::advanced::suggest_circuit_pattern(&a.query)
    }

    #[tool(description = "Rank candidate circuits by topological similarity to a reference.")]
    async fn find_similar_circuits(
        &self,
        Parameters(a): Parameters<SimilarArgs>,
    ) -> CallToolResult {
        tools::advanced::find_similar_circuits(&self.cli, &a.source, &a.candidates).await
    }

    // ── Library (Phase 3) ───────────────────────────────────────────────

    #[tool(description = "Validate a .alib library file or directory.")]
    async fn validate_library(&self, Parameters(a): Parameters<LibraryPathArg>) -> CallToolResult {
        tools::library::validate_library(&self.cli, &a.path).await
    }

    #[tool(description = "Compile SVG artwork into an Aelm symbol block.")]
    async fn compile_svg_to_symbol(&self, Parameters(a): Parameters<SvgArg>) -> CallToolResult {
        tools::library::compile_svg_to_symbol(&self.cli, &a.svg).await
    }
}

// ── Prompt router ───────────────────────────────────────────────────────────

#[prompt_router]
impl AelmMcpServer {
    #[prompt(
        name = "design_circuit",
        description = "Guide the design of a circuit from requirements to valid Aelm source."
    )]
    async fn design_circuit(&self, Parameters(a): Parameters<DesignArgs>) -> Vec<PromptMessage> {
        prompts::design_circuit(&a.description, a.constraints.as_deref())
    }

    #[prompt(
        name = "review_circuit",
        description = "Review an existing circuit for DRC issues, mistakes, and improvements."
    )]
    async fn review_circuit(&self, Parameters(a): Parameters<SourceArg>) -> Vec<PromptMessage> {
        prompts::review_circuit(&a.source)
    }

    #[prompt(
        name = "debug_drc",
        description = "Diagnose and fix DRC errors in a circuit, optionally focused on specific codes."
    )]
    async fn debug_drc(&self, Parameters(a): Parameters<DebugArgs>) -> Vec<PromptMessage> {
        prompts::debug_drc(&a.source, a.error_codes.as_deref())
    }

    #[prompt(
        name = "select_parts",
        description = "Recommend parts for a design requirement, grounded in the catalog."
    )]
    async fn select_parts(&self, Parameters(a): Parameters<RequirementsArg>) -> Vec<PromptMessage> {
        prompts::select_parts(&a.requirements)
    }

    #[prompt(
        name = "learn_aelm",
        description = "Structured Aelm tutorial at beginner, intermediate, or advanced level."
    )]
    async fn learn_aelm(&self, Parameters(a): Parameters<LevelArg>) -> Vec<PromptMessage> {
        prompts::learn_aelm(&a.level)
    }

    #[prompt(
        name = "explain_circuit",
        description = "Explain how a circuit works, step by step."
    )]
    async fn explain_circuit(&self, Parameters(a): Parameters<SourceArg>) -> Vec<PromptMessage> {
        prompts::explain_circuit(&a.source)
    }

    #[prompt(
        name = "interactive_design",
        description = "Iterative render → validate → refine design loop toward a goal."
    )]
    async fn interactive_design(&self, Parameters(a): Parameters<GoalArg>) -> Vec<PromptMessage> {
        prompts::interactive_design(&a.goal)
    }
}

// ── Server handler (info + resources) ───────────────────────────────────────

#[tool_handler(router = self.tool_router)]
#[prompt_handler(router = self.prompt_router)]
impl ServerHandler for AelmMcpServer {
    fn get_info(&self) -> ServerInfo {
        // ServerInfo is #[non_exhaustive]; build from Default and overwrite.
        let mut info = ServerInfo::default();
        info.protocol_version = ProtocolVersion::LATEST;
        info.capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .enable_prompts()
            .build();
        info.server_info = Implementation::from_build_env();
        info.instructions = Some(
            "Aelm MCP server: design, validate, render, and analyze text-based electronic \
             circuits. Use `get_dsl_reference` first to learn the DSL, `list_parts` / \
             `list_symbols` to discover the library, and `validate` / `render_svg` to check \
             your work."
                .to_string(),
        );
        info
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, ErrorData> {
        Ok(ListResourcesResult::with_all_items(resources::docs::list()))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, ErrorData> {
        match resources::docs::read(&request.uri) {
            Some(contents) => Ok(ReadResourceResult::new(vec![contents])),
            None => Err(ErrorData::resource_not_found(
                format!("unknown resource: {}", request.uri),
                None,
            )),
        }
    }
}
