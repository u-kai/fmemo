/// Response for GET /api/root - directory tree starting from root
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct DirectoryTree {
    pub path: String,
    pub files: Vec<String>,           // .fmemo file names
    pub subdirectories: Vec<DirectoryTree>,
}

/// Response for GET /api/files/{filepath} - file content
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct FileContent {
    pub memos: Vec<Memo>,
    pub last_modified: Option<u64>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Memo {
    level: Level,
    title: String,
    description: Option<String>,
    content: Option<String>,
    code_blocks: Vec<CodeBlock>,
    children: Vec<Memo>,
}

#[derive(Clone)]
pub struct MemoBuilder {
    level: Level,
    title: String,
    description: Option<String>,
    content: Option<String>,
    code_blocks: Vec<CodeBlock>,
    children: Vec<Memo>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct Level(u8);

impl Level {
    pub fn new(level: u8) -> Self {
        Self(level)
    }
    pub fn match_line(&self, line: &str) -> bool {
        line.starts_with(&"#".repeat(self.0 as usize + 1))
    }
    pub fn root() -> Self {
        Self(0)
    }
    pub fn child(&self) -> Self {
        Self(self.0 + 1)
    }
    pub fn level(&self) -> u8 {
        self.0
    }
}

impl MemoBuilder {
    pub fn new(level: Level, title: String) -> Self {
        Self {
            level,
            title,
            description: None,
            content: None,
            code_blocks: Vec::new(),
            children: Vec::new(),
        }
    }
    pub fn description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }
    pub fn content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    pub fn append_content(mut self, additional_content: &str) -> Self {
        let current = self.content.unwrap_or_default();
        self.content = Some(current + additional_content);
        self
    }
    pub fn add_code_block(mut self, language: String, code: String) -> Self {
        self.code_blocks.push(CodeBlock { language, code });
        self
    }
    pub fn add_child(mut self, child: Memo) -> Self {
        self.children.push(child);
        self
    }
    pub fn build(self) -> Memo {
        Memo {
            level: self.level,
            title: self.title,
            description: self.description,
            content: self.content,
            code_blocks: self.code_blocks,
            children: self.children,
        }
    }

    pub fn level(&self) -> &Level {
        &self.level
    }
}

impl Memo {
    pub fn level(&self) -> &Level {
        &self.level
    }

    pub fn title(&self) -> &String {
        &self.title
    }

    pub fn content(&self) -> &Option<String> {
        &self.content
    }

    pub fn description(&self) -> &Option<String> {
        &self.description
    }

    pub fn code_blocks(&self) -> &Vec<CodeBlock> {
        &self.code_blocks
    }

    pub fn children(&self) -> &Vec<Memo> {
        &self.children
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct CodeBlock {
    pub language: String,
    pub code: String,
}
