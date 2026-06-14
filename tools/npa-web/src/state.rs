use std::{
    error::Error,
    fmt,
    sync::{Mutex, MutexGuard},
};

use npa_api::{
    create_human_session, format_hash_string, get_human_state_by_id,
    human_api_default_compile_options, run_human_tactic, start_human_session_proof,
    verify_human_session, HumanCurrentModuleSource, HumanGoalId, HumanProofSessionStore,
    HumanProofStateStartError, HumanProofStateStartRequest, HumanSessionCreateError,
    HumanSessionCreateRequest, HumanSessionId, HumanSessionVerifyError, HumanSessionVerifyRequest,
    HumanStateApiError, HumanStateByIdRequest, HumanStateId, HumanStateRequestHeader,
    HumanTacticRunRequest, StructuredGoal, StructuredProofState,
};
use npa_cert::Name;
use npa_frontend::{
    parse_human_module, FileId, HumanDiagnostic, HumanDiagnosticSeverity, HumanItem,
};
use npa_tactic::TacticBudget;

use crate::{render, std_demo};

pub const DEFAULT_SOURCE: &str = "theorem id (A : Type) (x : A) : A := by exact x";
pub const DEFAULT_MODULE: &str = "Scratch";
pub const DEFAULT_THEOREM: &str = "Scratch.id";
pub const MAX_SOURCE_BYTES: usize = 128 * 1024;
pub const MAX_TACTIC_BYTES: usize = 4 * 1024;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DemoMode {
    #[default]
    ImportFree,
    Standard,
}

impl DemoMode {
    pub const ALL: [Self; 2] = [Self::ImportFree, Self::Standard];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ImportFree => "import-free",
            Self::Standard => "standard",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::ImportFree => "Import-free",
            Self::Standard => "Standard library",
        }
    }

    pub const fn default_source(self) -> &'static str {
        match self {
            Self::ImportFree => DEFAULT_SOURCE,
            Self::Standard => std_demo::STANDARD_DEMO_SOURCE,
        }
    }

    pub const fn default_module(self) -> &'static str {
        match self {
            Self::ImportFree => DEFAULT_MODULE,
            Self::Standard => std_demo::STANDARD_DEMO_MODULE,
        }
    }

    pub const fn default_theorem(self) -> &'static str {
        match self {
            Self::ImportFree => DEFAULT_THEOREM,
            Self::Standard => std_demo::STANDARD_DEMO_THEOREM,
        }
    }

    pub fn from_wire(value: &str) -> Option<Self> {
        match value {
            "import-free" => Some(Self::ImportFree),
            "standard" => Some(Self::Standard),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
pub struct WebState {
    human_store: Mutex<HumanProofSessionStore>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CreateSessionInput {
    pub demo: DemoMode,
    pub source: String,
    pub module: String,
    pub theorem: String,
}

impl Default for CreateSessionInput {
    fn default() -> Self {
        Self::for_demo(DemoMode::ImportFree)
    }
}

impl CreateSessionInput {
    pub fn for_demo(demo: DemoMode) -> Self {
        Self {
            demo,
            source: demo.default_source().to_owned(),
            module: demo.default_module().to_owned(),
            theorem: demo.default_theorem().to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RunTacticInput {
    pub session_id: String,
    pub document_id: String,
    pub document_version: String,
    pub state_id: String,
    pub goal_id: String,
    pub tactic: String,
}

impl RunTacticInput {
    pub fn for_workspace(workspace: &WebWorkspace, tactic: impl Into<String>) -> Self {
        Self {
            session_id: workspace.session_id.clone(),
            document_id: workspace.document_id.clone(),
            document_version: workspace.document_version.clone(),
            state_id: workspace.state_id.clone(),
            goal_id: workspace.goal_id.clone(),
            tactic: tactic.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerifyInput {
    pub session_id: String,
    pub document_id: String,
    pub document_version: String,
    pub state_id: String,
}

impl VerifyInput {
    pub fn for_workspace(workspace: &WebWorkspace) -> Self {
        Self {
            session_id: workspace.session_id.clone(),
            document_id: workspace.document_id.clone(),
            document_version: workspace.document_version.clone(),
            state_id: workspace.state_id.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebWorkspace {
    pub session_id: String,
    pub document_id: String,
    pub document_version: String,
    pub state_id: String,
    pub goal_id: String,
    pub tactic_input: String,
    pub goal: WebGoal,
    pub messages: Vec<WebMessage>,
    pub verify: WebVerify,
}

impl WebWorkspace {
    pub fn to_view(&self) -> render::WorkspaceView<'_> {
        render::WorkspaceView {
            session_id: &self.session_id,
            document_id: &self.document_id,
            document_version: &self.document_version,
            state_id: &self.state_id,
            goal_id: &self.goal_id,
            tactic_input: &self.tactic_input,
            goal: self.goal.to_view(),
            messages: render::MessagesView {
                items: self
                    .messages
                    .iter()
                    .map(WebMessage::to_view)
                    .collect::<Vec<_>>(),
            },
            verify: self.verify.to_view(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebGoal {
    pub has_goal: bool,
    pub label: String,
    pub context: Vec<WebBinding>,
    pub target: String,
}

impl WebGoal {
    fn empty() -> Self {
        Self {
            has_goal: false,
            label: String::new(),
            context: Vec::new(),
            target: String::new(),
        }
    }

    fn to_view(&self) -> render::GoalView<'_> {
        render::GoalView {
            has_goal: self.has_goal,
            label: &self.label,
            context: self
                .context
                .iter()
                .map(|binding| render::BindingView {
                    name: &binding.name,
                    ty: &binding.ty,
                })
                .collect(),
            target: &self.target,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebBinding {
    pub name: String,
    pub ty: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebMessage {
    pub severity: String,
    pub text: String,
}

impl WebMessage {
    fn info(text: impl Into<String>) -> Self {
        Self {
            severity: "info".to_owned(),
            text: text.into(),
        }
    }

    fn error(text: impl Into<String>) -> Self {
        Self {
            severity: "error".to_owned(),
            text: text.into(),
        }
    }

    fn to_view(&self) -> render::MessageView<'_> {
        render::MessageView {
            severity: &self.severity,
            text: &self.text,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebVerify {
    pub status: String,
    pub detail: String,
    pub root_decl_certificate_hash: String,
    pub certificate_hash: String,
    pub imports: Vec<WebImportSummary>,
}

impl WebVerify {
    fn pending() -> Self {
        Self {
            status: "not run".to_owned(),
            detail: "Verify after all goals are closed.".to_owned(),
            root_decl_certificate_hash: String::new(),
            certificate_hash: String::new(),
            imports: Vec::new(),
        }
    }

    pub fn to_view(&self) -> render::VerifyView<'_> {
        render::VerifyView {
            status: &self.status,
            detail: &self.detail,
            root_decl_certificate_hash: &self.root_decl_certificate_hash,
            certificate_hash: &self.certificate_hash,
            imports: self.imports.iter().map(WebImportSummary::to_view).collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebImportSummary {
    pub module: String,
    pub export_hash: String,
    pub certificate_hash: String,
    pub axiom_count: usize,
}

impl WebImportSummary {
    fn to_view(&self) -> render::VerifyImportView<'_> {
        render::VerifyImportView {
            module: &self.module,
            export_hash: &self.export_hash,
            certificate_hash: &self.certificate_hash,
            axiom_count: self.axiom_count,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WebFlowError {
    kind: WebFlowErrorKind,
    message: String,
}

impl WebFlowError {
    pub fn kind(&self) -> WebFlowErrorKind {
        self.kind
    }

    pub fn user_message(&self) -> &str {
        &self.message
    }

    fn new(kind: WebFlowErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

impl fmt::Display for WebFlowError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl Error for WebFlowError {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WebFlowErrorKind {
    SourceTooLarge,
    TacticTooLarge,
    UnsupportedImport,
    StandardDemoFixture,
    InvalidName,
    InvalidDocumentVersion,
    SessionStoreUnavailable,
    HumanSessionCreate,
    HumanProofStart,
    HumanStateLookup,
}

impl WebState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_session(&self, input: CreateSessionInput) -> Result<WebWorkspace, WebFlowError> {
        validate_source_input(&input.source, input.demo)?;
        let current_module = parse_canonical_name(&input.module, "module")?;
        let theorem_name = parse_canonical_name(&input.theorem, "theorem")?;
        let verified_modules = verified_modules_for_demo(input.demo)?;
        let mut store = self.lock_store()?;
        let created = create_human_session(
            &mut store,
            HumanSessionCreateRequest {
                current_module,
                current_source: HumanCurrentModuleSource {
                    file_id: FileId(0),
                    source: &input.source,
                },
                verified_modules: &verified_modules,
                imported_source_interfaces: &[],
                options: human_api_default_compile_options(),
            },
        )
        .map_err(map_create_error)?;
        let started = start_proof(
            &mut store,
            created.session_id.clone(),
            theorem_name,
            created.messages.clone(),
        )?;
        let header = HumanStateRequestHeader {
            session_id: created.session_id,
            document_id: created.document_id,
            document_version: created.document_version,
        };
        let state = state_by_id(&store, header.clone(), started.state_id)?;

        Ok(workspace_from_state(
            header,
            state,
            String::new(),
            Vec::new(),
        ))
    }

    pub fn run_tactic(&self, input: RunTacticInput) -> Result<WebWorkspace, WebFlowError> {
        validate_tactic_input(&input.tactic)?;
        let header = state_header_from_wire(
            &input.session_id,
            &input.document_id,
            &input.document_version,
        )?;
        let state_id = HumanStateId::new_unchecked(input.state_id);
        let goal_id = HumanGoalId::new_unchecked(input.goal_id);
        let mut store = self.lock_store()?;
        let response = run_human_tactic(
            &mut store,
            HumanTacticRunRequest {
                header: header.clone(),
                state_id: state_id.clone(),
                goal_id,
                tactic: input.tactic.clone(),
                budget: TacticBudget::default(),
            },
        );
        let next_state_id = response.new_state_id.clone().unwrap_or(state_id);
        let state = state_by_id(&store, header.clone(), next_state_id)?;
        let mut messages = diagnostic_messages(&response.messages);
        messages.push(WebMessage::info(format!(
            "tactic status: {}",
            response.status.as_str()
        )));
        if let Some(error) = response.error {
            messages.push(WebMessage::error(format!(
                "{}: {}",
                error.kind.as_str(),
                error.message
            )));
        }

        Ok(workspace_from_state(header, state, input.tactic, messages))
    }

    pub fn verify(&self, input: VerifyInput) -> Result<WebVerify, WebFlowError> {
        let header = state_header_from_wire(
            &input.session_id,
            &input.document_id,
            &input.document_version,
        )?;
        let state_id = HumanStateId::new_unchecked(input.state_id);
        let store = self.lock_store()?;
        match verify_human_session(
            &store,
            HumanSessionVerifyRequest {
                header,
                state_id: state_id.clone(),
            },
        ) {
            Ok(ok) => Ok(WebVerify {
                status: ok.status.as_str().to_owned(),
                detail: format!("{} verified.", ok.theorem_name.as_dotted()),
                root_decl_certificate_hash: format_hash_string(&ok.root_decl_certificate_hash),
                certificate_hash: format_hash_string(&ok.certificate_hash),
                imports: ok
                    .imports
                    .iter()
                    .map(|import| WebImportSummary {
                        module: import.module.as_dotted(),
                        export_hash: format_hash_string(&import.export_hash),
                        certificate_hash: format_hash_string(&import.certificate_hash),
                        axiom_count: import.module_axioms.len(),
                    })
                    .collect(),
            }),
            Err(HumanSessionVerifyError::OpenGoals { open_goals, .. }) => Ok(WebVerify {
                status: "open goals".to_owned(),
                detail: format_open_goals(&open_goals),
                root_decl_certificate_hash: String::new(),
                certificate_hash: String::new(),
                imports: Vec::new(),
            }),
            Err(HumanSessionVerifyError::CertificateHandoff { message, .. }) => Ok(WebVerify {
                status: "error".to_owned(),
                detail: message,
                root_decl_certificate_hash: String::new(),
                certificate_hash: String::new(),
                imports: Vec::new(),
            }),
            Err(HumanSessionVerifyError::State(error)) => Err(map_state_error(error)),
        }
    }

    fn lock_store(&self) -> Result<MutexGuard<'_, HumanProofSessionStore>, WebFlowError> {
        self.human_store.lock().map_err(|_| {
            WebFlowError::new(
                WebFlowErrorKind::SessionStoreUnavailable,
                "Human session store is unavailable.",
            )
        })
    }
}

fn verified_modules_for_demo(
    demo: DemoMode,
) -> Result<Vec<npa_cert::VerifiedModule>, WebFlowError> {
    match demo {
        DemoMode::ImportFree => Ok(Vec::new()),
        DemoMode::Standard => std_demo::load_standard_demo_verified_modules().map_err(|error| {
            WebFlowError::new(WebFlowErrorKind::StandardDemoFixture, error.user_message())
        }),
    }
}

fn start_proof(
    store: &mut HumanProofSessionStore,
    session_id: HumanSessionId,
    theorem_name: Name,
    messages: Vec<HumanDiagnostic>,
) -> Result<npa_api::HumanProofStateStartOk, WebFlowError> {
    start_human_session_proof(
        store,
        HumanProofStateStartRequest {
            session_id,
            theorem_name,
            source_span: None,
            selected_goal: None,
            messages,
        },
    )
    .map_err(map_start_error)
}

fn state_by_id(
    store: &HumanProofSessionStore,
    header: HumanStateRequestHeader,
    state_id: HumanStateId,
) -> Result<StructuredProofState, WebFlowError> {
    get_human_state_by_id(store, HumanStateByIdRequest { header, state_id })
        .map(|ok| ok.state)
        .map_err(map_state_error)
}

fn workspace_from_state(
    header: HumanStateRequestHeader,
    state: StructuredProofState,
    tactic_input: String,
    extra_messages: Vec<WebMessage>,
) -> WebWorkspace {
    let selected_goal = selected_goal(&state);
    let goal_id = selected_goal
        .as_ref()
        .map(|goal| goal.goal_id.wire().to_owned())
        .unwrap_or_default();
    let goal = selected_goal
        .map(goal_from_structured)
        .unwrap_or_else(WebGoal::empty);
    let mut messages = diagnostic_messages(&state.messages);
    messages.extend(extra_messages);

    WebWorkspace {
        session_id: header.session_id.wire().to_owned(),
        document_id: header.document_id.wire().to_owned(),
        document_version: header.document_version.as_u64().to_string(),
        state_id: state.state_id.wire().to_owned(),
        goal_id,
        tactic_input,
        goal,
        messages,
        verify: WebVerify::pending(),
    }
}

fn selected_goal(state: &StructuredProofState) -> Option<&StructuredGoal> {
    if let Some(goal_id) = state.selected_goal.as_ref() {
        state.goals.iter().find(|goal| &goal.goal_id == goal_id)
    } else {
        state.goals.first()
    }
}

fn goal_from_structured(goal: &StructuredGoal) -> WebGoal {
    WebGoal {
        has_goal: true,
        label: goal.goal_id.wire().to_owned(),
        context: goal
            .context
            .iter()
            .map(|hypothesis| WebBinding {
                name: hypothesis.name.clone(),
                ty: hypothesis.ty.pretty.clone(),
            })
            .collect(),
        target: goal.target.pretty.clone(),
    }
}

fn diagnostic_messages(diagnostics: &[HumanDiagnostic]) -> Vec<WebMessage> {
    diagnostics
        .iter()
        .map(|diagnostic| WebMessage {
            severity: match diagnostic.severity {
                HumanDiagnosticSeverity::Error => "error",
                HumanDiagnosticSeverity::Warning => "warning",
            }
            .to_owned(),
            text: diagnostic.message.clone(),
        })
        .collect()
}

fn validate_source_input(source: &str, demo: DemoMode) -> Result<(), WebFlowError> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err(WebFlowError::new(
            WebFlowErrorKind::SourceTooLarge,
            format!("Source input must be at most {MAX_SOURCE_BYTES} bytes."),
        ));
    }
    let imports = source_import_names(source);
    match demo {
        DemoMode::ImportFree if !imports.is_empty() => {
            return Err(WebFlowError::new(
                WebFlowErrorKind::UnsupportedImport,
                "Imports are disabled in the import-free demo.",
            ));
        }
        DemoMode::Standard => {
            if let Some(import) = imports
                .iter()
                .find(|import| !std_demo::STANDARD_DEMO_IMPORTS.contains(&import.as_str()))
            {
                return Err(WebFlowError::new(
                    WebFlowErrorKind::UnsupportedImport,
                    format!(
                        "The standard-library demo only allows fixed imports: {}. Unsupported import: {}.",
                        std_demo::STANDARD_DEMO_IMPORTS.join(", "),
                        if import.is_empty() { "<empty>" } else { import }
                    ),
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_tactic_input(tactic: &str) -> Result<(), WebFlowError> {
    if tactic.len() > MAX_TACTIC_BYTES {
        return Err(WebFlowError::new(
            WebFlowErrorKind::TacticTooLarge,
            format!("Tactic input must be at most {MAX_TACTIC_BYTES} bytes."),
        ));
    }
    Ok(())
}

fn source_import_names(source: &str) -> Vec<String> {
    let mut imports = source_import_line_names(source);
    imports.extend(parsed_source_import_names(source));
    imports.sort();
    imports.dedup();
    imports
}

fn source_import_line_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let rest = line.strip_prefix("import")?;
            let starts_import_keyword = rest.is_empty()
                || rest
                    .chars()
                    .next()
                    .map(|character| character.is_whitespace())
                    .unwrap_or(false);
            if starts_import_keyword {
                Some(
                    rest.split_whitespace()
                        .map(str::to_owned)
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .flat_map(|names| {
            if names.is_empty() {
                vec![String::new()]
            } else {
                names
            }
        })
        .collect()
}

fn parsed_source_import_names(source: &str) -> Vec<String> {
    parse_human_module(FileId(0), source)
        .map(|module| {
            module
                .items
                .iter()
                .filter_map(|item| match item {
                    HumanItem::Import { module, .. } => Some(module.as_dotted()),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn parse_canonical_name(value: &str, field: &'static str) -> Result<Name, WebFlowError> {
    let name = Name::from_dotted(value);
    if name.is_canonical() {
        Ok(name)
    } else {
        Err(WebFlowError::new(
            WebFlowErrorKind::InvalidName,
            format!("{field} must be a canonical dotted NPA name."),
        ))
    }
}

fn state_header_from_wire(
    session_id: &str,
    document_id: &str,
    document_version: &str,
) -> Result<HumanStateRequestHeader, WebFlowError> {
    Ok(HumanStateRequestHeader {
        session_id: npa_api::HumanSessionId::new_unchecked(session_id),
        document_id: npa_api::HumanDocumentId::new_unchecked(document_id),
        document_version: parse_document_version(document_version)?,
    })
}

fn parse_document_version(value: &str) -> Result<npa_api::HumanDocumentVersion, WebFlowError> {
    let parsed = value.parse::<u64>().map_err(|_| {
        WebFlowError::new(
            WebFlowErrorKind::InvalidDocumentVersion,
            "Document version must be an unsigned integer.",
        )
    })?;
    if parsed == 0 {
        return Err(WebFlowError::new(
            WebFlowErrorKind::InvalidDocumentVersion,
            "Document version must be greater than zero.",
        ));
    }
    Ok(npa_api::HumanDocumentVersion::new_unchecked(parsed))
}

fn format_open_goals(open_goals: &[HumanGoalId]) -> String {
    if open_goals.is_empty() {
        "Open goals remain.".to_owned()
    } else {
        format!(
            "Open goals: {}.",
            open_goals
                .iter()
                .map(|goal_id| goal_id.wire())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

fn map_create_error(error: HumanSessionCreateError) -> WebFlowError {
    match error {
        HumanSessionCreateError::IdSpaceExhausted => WebFlowError::new(
            WebFlowErrorKind::HumanSessionCreate,
            "Human session id space is exhausted.",
        ),
    }
}

fn map_start_error(error: HumanProofStateStartError) -> WebFlowError {
    let message = match error {
        HumanProofStateStartError::UnknownSession { .. } => "Unknown Human session.".to_owned(),
        HumanProofStateStartError::IdSpaceExhausted => {
            "Human proof state id space is exhausted.".to_owned()
        }
        HumanProofStateStartError::Start(start_error) => match start_error {
            npa_api::HumanStartProofError::Human(error) => error.diagnostic.message,
            npa_api::HumanStartProofError::Machine(diagnostic) => diagnostic.message.to_string(),
        },
    };
    WebFlowError::new(WebFlowErrorKind::HumanProofStart, message)
}

fn map_state_error(error: HumanStateApiError) -> WebFlowError {
    WebFlowError::new(
        WebFlowErrorKind::HumanStateLookup,
        format!("Human proof state lookup failed: {error:?}."),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_flow_default_proof_advances_and_verifies() {
        let state = WebState::new();
        let created = state
            .create_session(CreateSessionInput::default())
            .expect("default session should start");

        assert!(created.goal.has_goal);
        assert!(!created.goal_id.is_empty());

        let after_intro_a = state
            .run_tactic(RunTacticInput::for_workspace(&created, "intro A"))
            .expect("intro A should run");
        assert!(after_intro_a.goal.has_goal);

        let after_intro_x = state
            .run_tactic(RunTacticInput::for_workspace(&after_intro_a, "intro x"))
            .expect("intro x should run");
        assert!(after_intro_x.goal.has_goal);

        let after_exact = state
            .run_tactic(RunTacticInput::for_workspace(&after_intro_x, "exact x"))
            .expect("exact x should run");
        assert!(!after_exact.goal.has_goal);
        assert!(after_exact.goal_id.is_empty());

        let verified = state
            .verify(VerifyInput::for_workspace(&after_exact))
            .expect("closed default proof should verify");
        assert_eq!(verified.status, "verified");
        assert!(!verified.root_decl_certificate_hash.is_empty());
        assert!(!verified.certificate_hash.is_empty());
        assert!(verified.imports.is_empty());
    }

    #[test]
    fn std_demo_default_proof_advances_and_verifies_with_fixed_imports() {
        let state = WebState::new();
        let created = state
            .create_session(CreateSessionInput::for_demo(DemoMode::Standard))
            .expect("standard demo session should start");

        assert!(created.goal.has_goal);

        let after_intro = state
            .run_tactic(RunTacticInput::for_workspace(&created, "intro n"))
            .expect("intro n should run");
        assert!(after_intro.goal.has_goal);

        let after_exact = state
            .run_tactic(RunTacticInput::for_workspace(
                &after_intro,
                "exact @Eq.refl.{1} Nat n",
            ))
            .expect("Eq.refl should close the standard demo");
        assert!(!after_exact.goal.has_goal);

        let verified = state
            .verify(VerifyInput::for_workspace(&after_exact))
            .expect("closed standard demo should verify");
        let imports = verified
            .imports
            .iter()
            .map(|import| import.module.as_str())
            .collect::<Vec<_>>();

        assert_eq!(verified.status, "verified");
        assert_eq!(imports, vec!["Std.Logic.Eq", "Std.Nat.Basic"]);
        assert!(verified
            .imports
            .iter()
            .all(|import| !import.export_hash.is_empty() && !import.certificate_hash.is_empty()));
    }

    #[test]
    fn human_flow_rejects_source_over_128_kib() {
        let state = WebState::new();
        let input = CreateSessionInput {
            source: "x".repeat(MAX_SOURCE_BYTES + 1),
            ..CreateSessionInput::default()
        };

        let error = state
            .create_session(input)
            .expect_err("oversized source should be rejected");

        assert_eq!(error.kind(), WebFlowErrorKind::SourceTooLarge);
        assert!(error.user_message().contains("Source input"));
    }

    #[test]
    fn human_flow_rejects_tactic_over_4_kib() {
        let state = WebState::new();
        let created = state
            .create_session(CreateSessionInput::default())
            .expect("default session should start");
        let input = RunTacticInput::for_workspace(&created, "x".repeat(MAX_TACTIC_BYTES + 1));

        let error = state
            .run_tactic(input)
            .expect_err("oversized tactic should be rejected");

        assert_eq!(error.kind(), WebFlowErrorKind::TacticTooLarge);
        assert!(error.user_message().contains("Tactic input"));
    }

    #[test]
    fn human_flow_rejects_browser_imports() {
        let state = WebState::new();
        let input = CreateSessionInput {
            source: "\timport\tStd.Nat.Basic\ntheorem id (A : Type) (x : A) : A := by exact x"
                .to_owned(),
            ..CreateSessionInput::default()
        };

        let error = state
            .create_session(input)
            .expect_err("imports should be rejected before session creation");

        assert_eq!(error.kind(), WebFlowErrorKind::UnsupportedImport);
    }

    #[test]
    fn std_demo_rejects_imports_outside_fixed_allowlist() {
        let state = WebState::new();
        let input = CreateSessionInput {
            source: "import Std.Nat.Basic\nimport Proofs.Ai.Bad\ntheorem bad : Type := Type"
                .to_owned(),
            module: "BadDemo".to_owned(),
            theorem: "BadDemo.bad".to_owned(),
            ..CreateSessionInput::for_demo(DemoMode::Standard)
        };

        let error = state
            .create_session(input)
            .expect_err("standard demo should reject unowned imports");

        assert_eq!(error.kind(), WebFlowErrorKind::UnsupportedImport);
        assert!(error.user_message().contains("Proofs.Ai.Bad"));
    }

    #[test]
    fn human_flow_rejects_path_like_names() {
        let state = WebState::new();
        let input = CreateSessionInput {
            module: "../Scratch".to_owned(),
            ..CreateSessionInput::default()
        };

        let error = state
            .create_session(input)
            .expect_err("path-like module should be rejected");

        assert_eq!(error.kind(), WebFlowErrorKind::InvalidName);
    }

    #[test]
    fn human_flow_verify_reports_open_goals_as_user_visible_status() {
        let state = WebState::new();
        let created = state
            .create_session(CreateSessionInput::default())
            .expect("default session should start");

        let verify = state
            .verify(VerifyInput::for_workspace(&created))
            .expect("open-goal verification should be user-facing");

        assert_eq!(verify.status, "open goals");
        assert!(verify.detail.contains("hgoal_"));
        assert!(verify.certificate_hash.is_empty());
    }

    #[test]
    fn human_flow_workspace_converts_to_render_view() {
        let state = WebState::new();
        let workspace = state
            .create_session(CreateSessionInput::default())
            .expect("default session should start");

        let view = workspace.to_view();

        assert_eq!(view.session_id, workspace.session_id);
        assert_eq!(view.goal.has_goal, workspace.goal.has_goal);
        assert_eq!(view.verify.status, "not run");
        assert!(view.verify.imports.is_empty());
    }
}
