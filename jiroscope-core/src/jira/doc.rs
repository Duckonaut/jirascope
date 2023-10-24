use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AtlassianDoc {
    pub version: isize,
    #[serde(rename = "type")]
    pub type_: String,
    pub content: Vec<Content>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Mark {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<MarkAttrs>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MarkAttrs {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<u8>,
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

    pub fn from_markdown(markdown: &str) -> Self {
        let mut doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: Vec::new(),
        };

        // our markdown settings don't error, so we can unwrap
        let md_ast = markdown::to_mdast(markdown, &markdown::ParseOptions::gfm()).unwrap();

        if let markdown::mdast::Node::Root(root) = md_ast {
            for child in root.children {
                doc.content
                    .extend(Content::from_markdown_node(child).flatten());
            }
        }

        doc
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

    pub fn from_markdown_node(node: markdown::mdast::Node) -> Self {
        match node {
            markdown::mdast::Node::BlockQuote(bq) => {
                let mut content = Vec::new();
                for child in bq.children {
                    content.push(Content::from_markdown_node(child));
                }
                Content {
                    type_: "blockquote".to_string(),
                    content: Some(content),
                    text: None,
                    marks: None,
                }
            }
            markdown::mdast::Node::List(l) => {
                let mut content = Vec::new();
                for child in l.children {
                    content.push(Content::from_markdown_node(child));
                }
                if l.ordered {
                    Content {
                        type_: "orderedList".to_string(),
                        content: Some(content),
                        text: None,
                        marks: None,
                    }
                } else {
                    Content {
                        type_: "bulletList".to_string(),
                        content: Some(content),
                        text: None,
                        marks: None,
                    }
                }
            }
            markdown::mdast::Node::Break(_) => Content {
                type_: "hardBreak".to_string(),
                content: None,
                text: None,
                marks: None,
            },
            markdown::mdast::Node::InlineCode(ic) => Content {
                type_: "code".to_string(),
                content: None,
                text: Some(ic.value),
                marks: Some(vec![Mark {
                    type_: "code".to_string(),
                    attrs: None,
                }]),
            },
            markdown::mdast::Node::Delete(d) => {
                let mut content = Vec::new();
                for child in d.children {
                    content.push(Content::from_markdown_node(child));
                }
                Content {
                    type_: "text".to_string(),
                    content: Some(content),
                    text: None,
                    marks: Some(vec![Mark {
                        type_: "strike".to_string(),
                        attrs: None,
                    }]),
                }
            }
            markdown::mdast::Node::Emphasis(emp) => {
                let mut content = Vec::new();
                for child in emp.children {
                    content.push(Content::from_markdown_node(child));
                }
                Content {
                    type_: "text".to_string(),
                    content: Some(content),
                    text: None,
                    marks: Some(vec![Mark {
                        type_: "em".to_string(),
                        attrs: None,
                    }]),
                }
            }
            markdown::mdast::Node::Link(l) => {
                let mut content = Vec::new();
                for child in l.children {
                    content.push(Content::from_markdown_node(child));
                }
                Content {
                    type_: "text".to_string(),
                    content: Some(content),
                    text: None,
                    marks: Some(vec![Mark {
                        type_: "link".to_string(),
                        attrs: Some(MarkAttrs {
                            href: Some(l.url),
                            title: l.title,
                            level: None,
                        }),
                    }]),
                }
            }
            markdown::mdast::Node::Strong(s) => {
                let mut content = Vec::new();
                for child in s.children {
                    content.push(Content::from_markdown_node(child));
                }
                Content {
                    type_: "text".to_string(),
                    content: Some(content),
                    text: None,
                    marks: Some(vec![Mark {
                        type_: "strong".to_string(),
                        attrs: None,
                    }]),
                }
            }
            markdown::mdast::Node::Text(t) => Content {
                type_: "text".to_string(),
                content: None,
                text: Some(t.value),
                marks: None,
            },
            markdown::mdast::Node::Code(c) => Content {
                type_: "text".to_string(),
                content: None,
                text: Some(c.value),
                marks: Some(vec![Mark {
                    type_: "code".to_string(),
                    attrs: None,
                }]),
            },
            markdown::mdast::Node::Heading(h) => {
                let mut content = Vec::new();
                for child in h.children {
                    content.push(Content::from_markdown_node(child));
                }
                Content {
                    type_: "heading".to_string(),
                    content: Some(content),
                    text: None,
                    marks: Some(vec![Mark {
                        type_: format!("heading{}", h.depth),
                        attrs: Some(MarkAttrs {
                            href: None,
                            title: None,
                            level: Some(h.depth),
                        }),
                    }]),
                }
            }
            markdown::mdast::Node::Table(table) => {
                let mut content = Vec::new();
                for child in table.children {
                    content.push(Content::from_markdown_node(child));
                }
                Content {
                    type_: "table".to_string(),
                    content: Some(content),
                    text: None,
                    marks: None,
                }
            }
            markdown::mdast::Node::ThematicBreak(_) => Content {
                type_: "rule".to_string(),
                content: None,
                text: None,
                marks: None,
            },
            markdown::mdast::Node::TableRow(tr) => {
                let mut content = Vec::new();
                for child in tr.children {
                    content.push(Content::from_markdown_node(child));
                }
                Content {
                    type_: "tableRow".to_string(),
                    content: Some(content),
                    text: None,
                    marks: None,
                }
            }
            markdown::mdast::Node::TableCell(tc) => {
                let mut content = Vec::new();
                for child in tc.children {
                    content.push(Content::from_markdown_node(child));
                }
                Content {
                    type_: "tableCell".to_string(),
                    content: Some(content),
                    text: None,
                    marks: None,
                }
            }
            markdown::mdast::Node::ListItem(li) => {
                let mut content = Vec::new();
                for child in li.children {
                    content.push(Content::from_markdown_node(child));
                }
                Content {
                    type_: "listItem".to_string(),
                    content: Some(content),
                    text: None,
                    marks: None,
                }
            }
            markdown::mdast::Node::Paragraph(p) => {
                let mut content = Vec::new();
                for child in p.children {
                    content.push(Content::from_markdown_node(child));
                }
                Content {
                    type_: "paragraph".to_string(),
                    content: Some(content),
                    text: None,
                    marks: None,
                }
            }
            _ => Content::text("-!- unimplemented markdown node -!-"),
        }
    }

    /// Flatten the inline nodes like 'text' and 'code' so they are not nested
    /// Turns leaves of the tree into the content of the parent, collecting
    /// marks from the path to the leaf
    ///
    /// Turns:
    /// ```json
    /// {
    ///  "type": "paragraph",
    ///  "content": [
    ///     {
    ///         "type": "text",
    ///         "text": null,
    ///         "content": [
    ///             {
    ///                 "type": "text",
    ///                 "text": "This is ",
    ///                 "content": [
    ///                     {
    ///                         "type": "text",
    ///                         "text": "emphasized bold",
    ///                         "content": null,
    ///                         "marks": [
    ///                             {
    ///                                 "type": "strong",
    ///                             }
    ///                         ]
    ///                     }
    ///                 ],
    ///                 "marks": null,
    ///             },
    ///         ],
    ///         "marks": [
    ///             {
    ///                 "type": "em",
    ///             }
    ///         ]
    ///     }
    /// ]
    /// }
    /// ```
    /// Into:
    /// ```json
    /// {
    ///   "type": "paragraph",
    ///   "content": [
    ///     {
    ///       "type": "text",
    ///       "text": "This is ",
    ///       "content": null,
    ///       "marks": [
    ///         {
    ///           "type": "em"
    ///         },
    ///       ],
    ///     },
    ///     {
    ///       "type": "text",
    ///       "text": "emphasized bold",
    ///       "content": null,
    ///       "marks": [
    ///         {
    ///           "type": "em"
    ///         },
    ///         {
    ///           "type": "strong"
    ///         }
    ///       ]
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn flatten(mut self) -> Vec<Self> {
        println!("Flattening: {:?}", self);
        if self.inline() && self.content.is_some() {
            let mut flattened = Vec::new();
            for child in self.content.unwrap() {
                let c = child.flatten();

                if let Some(marks) = &self.marks {
                    for mut c in c {
                        if c.marks.is_some() {
                            c.marks.as_mut().unwrap().extend(marks.clone());
                        } else {
                            c.marks = Some(marks.clone());
                        }
                        flattened.push(c);
                    }
                } else {
                    flattened.extend(c);
                }
            }
            dbg!(&flattened);
            flattened
        } else {
            if let Some(content) = self.content {
                let mut flattened = Vec::new();
                for child in content {
                    flattened.extend(child.flatten());
                }
                self.content = Some(flattened);
            }

            vec![self]
        }
    }

    /// Returns true if the content is inline
    pub fn inline(&self) -> bool {
        matches!(
            self.type_.as_str(),
            "text" | "code" | "inlineCard" | "mention" | "emoji"
        )
    }
}
