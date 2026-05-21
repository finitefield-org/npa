use crate::{FileId, Span};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HumanModule {
    pub file_id: FileId,
    pub items: Vec<HumanItem>,
    pub span: Span,
}

impl HumanModule {
    pub fn empty(file_id: FileId, source_len: u32) -> Self {
        Self {
            file_id,
            items: Vec::new(),
            span: Span::new(file_id, 0, source_len),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HumanItem {}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HumanExpr {}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct HumanCompileOptions {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn human_module_is_distinct_from_machine_module() {
        let module = HumanModule::empty(FileId(3), 11);

        assert_eq!(module.file_id, FileId(3));
        assert!(module.items.is_empty());
        assert_eq!(module.span, Span::new(FileId(3), 0, 11));
    }
}
