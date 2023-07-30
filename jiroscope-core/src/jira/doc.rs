use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlassianDoc {
    pub version: isize,
    #[serde(rename = "type")]
    pub type_: String,
    pub content: Vec<Content>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<Content>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marks: Option<Vec<Mark>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mark {
    #[serde(rename = "type")]
    pub type_: String,
}

impl AtlassianDoc {
    pub fn text(text: &str) -> Self {
        AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content::paragraph(text)],
        }
    }

    pub fn to_markdown(&self) -> String {
        let mut markdown = String::new();
        for content in &self.content {
            markdown.push_str(&content.to_markdown());
        }
        markdown
    }
}

impl Content {
    pub fn text(text: &str) -> Self {
        Content {
            type_: "text".to_string(),
            content: None,
            text: Some(text.to_string()),
            marks: None,
        }
    }

    pub fn paragraph(text: &str) -> Self {
        Content {
            type_: "paragraph".to_string(),
            content: Some(vec![Content::text(text)]),
            text: None,
            marks: None,
        }
    }

    pub fn to_markdown(&self) -> String {
        // TODO: handle marks
        let mut markdown = String::new();
        if let Some(text) = &self.text {
            markdown.push_str(text);
        }
        if let Some(content) = &self.content {
            for content in content {
                markdown.push_str(&content.to_markdown());
            }
        }
        markdown
    }
}
