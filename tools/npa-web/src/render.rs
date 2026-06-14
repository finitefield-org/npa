use std::{error::Error, fmt};

use go_html_template::{Template, TemplateError};
use serde::Serialize;

const PAGE_TEMPLATE: &str = include_str!("../templates/page.html");
const WORKSPACE_TEMPLATE: &str = include_str!("../templates/workspace.html");
const GOAL_TEMPLATE: &str = include_str!("../templates/goal.html");
const MESSAGES_TEMPLATE: &str = include_str!("../templates/messages.html");
const VERIFY_TEMPLATE: &str = include_str!("../templates/verify.html");

pub(crate) const TEMPLATE_SOURCES: &[&str] = &[
    PAGE_TEMPLATE,
    WORKSPACE_TEMPLATE,
    GOAL_TEMPLATE,
    MESSAGES_TEMPLATE,
    VERIFY_TEMPLATE,
];

pub struct Renderer {
    template: Template,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateName {
    Page,
    Workspace,
    Goal,
    Messages,
    Verify,
}

impl TemplateName {
    fn as_str(self) -> &'static str {
        match self {
            TemplateName::Page => "page",
            TemplateName::Workspace => "workspace",
            TemplateName::Goal => "goal",
            TemplateName::Messages => "messages",
            TemplateName::Verify => "verify",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderError {
    template: &'static str,
    phase: RenderErrorPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderErrorPhase {
    Parse,
    Execute,
}

impl RenderError {
    fn parse(template: &'static str, _source: TemplateError) -> Self {
        Self {
            template,
            phase: RenderErrorPhase::Parse,
        }
    }

    fn execute(template: &'static str, _source: TemplateError) -> Self {
        Self {
            template,
            phase: RenderErrorPhase::Execute,
        }
    }

    pub fn user_message(&self) -> &'static str {
        match self.phase {
            RenderErrorPhase::Parse => "template parse failed",
            RenderErrorPhase::Execute => "template render failed",
        }
    }

    pub fn template(&self) -> &'static str {
        self.template
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.template, self.user_message())
    }
}

impl Error for RenderError {}

impl Renderer {
    pub fn new() -> Result<Self, RenderError> {
        Self::from_source(&template_bundle())
    }

    fn from_source(source: &str) -> Result<Self, RenderError> {
        let template = Template::new(TemplateName::Page.as_str())
            .option("missingkey=error")
            .map_err(|error| RenderError::parse("templates", error))?
            .parse(source)
            .map_err(|error| RenderError::parse("templates", error))?;
        Ok(Self { template })
    }

    pub fn render_page(&self, view: &PageView<'_>) -> Result<String, RenderError> {
        self.render(TemplateName::Page, view)
    }

    pub fn render_workspace(&self, view: &WorkspaceView<'_>) -> Result<String, RenderError> {
        self.render(TemplateName::Workspace, view)
    }

    pub fn render_goal(&self, view: &GoalView<'_>) -> Result<String, RenderError> {
        self.render(TemplateName::Goal, view)
    }

    pub fn render_messages(&self, view: &MessagesView<'_>) -> Result<String, RenderError> {
        self.render(TemplateName::Messages, view)
    }

    pub fn render_verify(&self, view: &VerifyView<'_>) -> Result<String, RenderError> {
        self.render(TemplateName::Verify, view)
    }

    fn render<T: Serialize>(&self, name: TemplateName, view: &T) -> Result<String, RenderError> {
        self.template
            .execute_template_to_string(name.as_str(), view)
            .map_err(|error| RenderError::execute(name.as_str(), error))
    }
}

fn template_bundle() -> String {
    TEMPLATE_SOURCES.join("\n")
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct PageView<'a> {
    pub title: &'a str,
    pub source: &'a str,
    pub module_name: &'a str,
    pub theorem_name: &'a str,
    pub workspace: WorkspaceView<'a>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct WorkspaceView<'a> {
    pub session_id: &'a str,
    pub document_id: &'a str,
    pub document_version: &'a str,
    pub state_id: &'a str,
    pub goal_id: &'a str,
    pub tactic_input: &'a str,
    pub goal: GoalView<'a>,
    pub messages: MessagesView<'a>,
    pub verify: VerifyView<'a>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct GoalView<'a> {
    pub has_goal: bool,
    pub label: &'a str,
    pub context: Vec<BindingView<'a>>,
    pub target: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct BindingView<'a> {
    pub name: &'a str,
    pub ty: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessagesView<'a> {
    pub items: Vec<MessageView<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct MessageView<'a> {
    pub severity: &'a str,
    pub text: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct VerifyView<'a> {
    pub status: &'a str,
    pub detail: &'a str,
    pub certificate_hash: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn render_page_escapes_user_source_textarea_content() {
        let renderer = Renderer::new().expect("renderer should parse");
        let view = sample_page_view("theorem bad : Type := <tag> & \"quote\"", "", "");

        let html = renderer
            .render_page(&view)
            .expect("page should render with escaped source");

        assert!(html.contains("theorem bad : Type := &lt;tag&gt; &amp; &#34;quote&#34;"));
        assert!(!html.contains("<tag> & \"quote\""));
    }

    #[test]
    fn render_page_neutralizes_textarea_breakout_source() {
        let renderer = Renderer::new().expect("renderer should parse");
        let view = sample_page_view("</textarea><script>alert(1)</script>", "", "");

        let html = renderer
            .render_page(&view)
            .expect("page should render with neutralized source");

        assert!(!html.contains("</textarea><script>"));
        assert!(!html.contains("<script>alert(1)</script>"));
    }

    #[test]
    fn render_workspace_escapes_tactic_input_attribute_content() {
        let renderer = Renderer::new().expect("renderer should parse");
        let mut workspace = sample_workspace_view();
        workspace.tactic_input = "\" autofocus onfocus=\"alert(1)";

        let html = renderer
            .render_workspace(&workspace)
            .expect("workspace should render with escaped tactic input");

        assert!(html.contains("value=\"&#34; autofocus onfocus=&#34;alert(1)\""));
        assert!(!html.contains("autofocus onfocus=\"alert(1)"));
    }

    #[test]
    fn render_messages_escapes_diagnostics() {
        let renderer = Renderer::new().expect("renderer should parse");
        let view = MessagesView {
            items: vec![MessageView {
                severity: "error",
                text: "<b>bad tactic</b> & retry",
            }],
        };

        let html = renderer
            .render_messages(&view)
            .expect("messages should render with escaped diagnostics");

        assert!(html.contains("&lt;b&gt;bad tactic&lt;/b&gt; &amp; retry"));
        assert!(!html.contains("<b>bad tactic</b>"));
    }

    #[test]
    fn render_goal_escapes_context_and_target() {
        let renderer = Renderer::new().expect("renderer should parse");
        let view = GoalView {
            has_goal: true,
            label: "goal <1>",
            context: vec![BindingView {
                name: "x<script>",
                ty: "A & B",
            }],
            target: "</section>",
        };

        let html = renderer
            .render_goal(&view)
            .expect("goal should render with escaped values");

        assert!(html.contains("goal &lt;1&gt;"));
        assert!(html.contains("x&lt;script&gt;"));
        assert!(html.contains("A &amp; B"));
        assert!(html.contains("&lt;/section&gt;"));
    }

    #[test]
    fn render_verify_escapes_certificate_display_fields() {
        let renderer = Renderer::new().expect("renderer should parse");
        let view = VerifyView {
            status: "verified <ok>",
            detail: "hash & imports",
            certificate_hash: "sha256:<bad>",
        };

        let html = renderer
            .render_verify(&view)
            .expect("verify should render with escaped fields");

        assert!(html.contains("verified &lt;ok&gt;"));
        assert!(html.contains("hash &amp; imports"));
        assert!(html.contains("sha256:&lt;bad&gt;"));
    }

    #[test]
    fn render_error_message_is_short_and_sanitized() {
        let renderer = Renderer::from_source(r#"{{define "page"}}{{.Missing}}{{end}}"#)
            .expect("test template should parse");

        let error = renderer
            .render(TemplateName::Page, &json!({}))
            .expect_err("missing key should be converted");

        assert_eq!(error.user_message(), "template render failed");
        assert_eq!(error.to_string(), "page: template render failed");
        let formatted = error.to_string();
        assert!(!formatted.contains(env!("CARGO_MANIFEST_DIR")));
        assert!(!formatted.contains("panicked"));
        assert!(!formatted.contains("Missing"));
    }

    #[test]
    fn parse_error_message_is_short_and_sanitized() {
        let error = match Renderer::from_source(r#"{{define "page"}}{{if .Open}}"#) {
            Ok(_) => panic!("bad template should fail during parsing"),
            Err(error) => error,
        };

        assert_eq!(error.user_message(), "template parse failed");
        let formatted = error.to_string();
        assert!(!formatted.contains(env!("CARGO_MANIFEST_DIR")));
        assert!(!formatted.contains("panicked"));
        assert!(!formatted.contains("Open"));
    }

    fn sample_page_view<'a>(
        source: &'a str,
        tactic_input: &'a str,
        diagnostic: &'a str,
    ) -> PageView<'a> {
        PageView {
            title: "NPA Web",
            source,
            module_name: "Scratch",
            theorem_name: "Scratch.id",
            workspace: WorkspaceView {
                tactic_input,
                messages: MessagesView {
                    items: if diagnostic.is_empty() {
                        Vec::new()
                    } else {
                        vec![MessageView {
                            severity: "info",
                            text: diagnostic,
                        }]
                    },
                },
                ..sample_workspace_view()
            },
        }
    }

    fn sample_workspace_view<'a>() -> WorkspaceView<'a> {
        WorkspaceView {
            session_id: "sess_1",
            document_id: "doc_1",
            document_version: "1",
            state_id: "state_1",
            goal_id: "goal_1",
            tactic_input: "intro A",
            goal: GoalView {
                has_goal: true,
                label: "goal_1",
                context: vec![BindingView {
                    name: "A",
                    ty: "Type",
                }],
                target: "A",
            },
            messages: MessagesView { items: Vec::new() },
            verify: VerifyView {
                status: "not verified",
                detail: "Run verify after closing all goals.",
                certificate_hash: "",
            },
        }
    }
}
