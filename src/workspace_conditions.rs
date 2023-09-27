use std::path::Path;

pub trait WorkspaceCondition {
    fn meets_condition(&self, path: &Path) -> bool;
}

pub struct HasAnyFileCondition {
    pub files: Vec<String>,
}

impl WorkspaceCondition for HasAnyFileCondition {
    fn meets_condition(&self, path: &Path) -> bool {
        for file in &self.files {
            if path.join(file).exists() {
                return true;
            }
        }
        false
    }
}

pub struct HasAllFilesCondition {
    pub files: Vec<String>,
}

impl WorkspaceCondition for HasAllFilesCondition {
    fn meets_condition(&self, path: &Path) -> bool {
        for file in &self.files {
            if !path.join(file).exists() {
                return false;
            }
        }
        true
    }
}

/// A condition that always returns true, used as a default condition if no others
/// are specified.
pub struct NullCondition {}

impl WorkspaceCondition for NullCondition {
    fn meets_condition(&self, _path: &Path) -> bool {
        true
    }
}
