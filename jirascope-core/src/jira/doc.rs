use std::collections::HashMap;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attrs: Option<HashMap<String, String>>,
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
            content.to_markdown(&mut markdown).unwrap();
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
            attrs: None,
        }
    }

    pub fn paragraph(text: &str) -> Self {
        Content {
            type_: "paragraph".to_string(),
            content: Some(vec![Content::text(text)]),
            text: None,
            marks: None,
            attrs: None,
        }
    }

    fn marks_start<W: std::fmt::Write>(&self, writer: &mut W) -> std::fmt::Result {
        if let Some(marks_) = &self.marks {
            for mark in marks_ {
                match mark.type_.as_str() {
                    "strong" => write!(writer, "**")?,
                    "em" => write!(writer, "*")?,
                    "strike" => write!(writer, "~~")?,
                    "code" => write!(writer, "`")?,
                    "link" => {
                        write!(writer, "[")?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn marks_end<W: std::fmt::Write>(&self, writer: &mut W) -> std::fmt::Result {
        if let Some(marks_) = &self.marks {
            for mark in marks_.iter().rev() {
                match mark.type_.as_str() {
                    "strong" => write!(writer, "**")?,
                    "em" => write!(writer, "*")?,
                    "strike" => write!(writer, "~~")?,
                    "code" => write!(writer, "`")?,
                    "link" => {
                        write!(writer, "](")?;
                        if let Some(attrs) = &mark.attrs {
                            if let Some(href) = &attrs.href {
                                writer.write_str(href)?;
                            }

                            if let Some(title) = &attrs.title {
                                write!(writer, " \"{}\"", title)?;
                            }
                        }
                        write!(writer, ")")?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub fn to_markdown<W: std::fmt::Write>(&self, writer: &mut W) -> std::fmt::Result {
        match self.type_.as_str() {
            "codeBlock" => {
                write!(writer, "```")?;
                if let Some(attrs) = &self.attrs {
                    if let Some(lang) = attrs.get("language") {
                        writeln!(writer, "{}", lang)?;
                    }
                }
                if let Some(text) = &self.text {
                    writeln!(writer, "{}", text)?;
                }
                writeln!(writer, "```")
            }
            "blockquote" => {
                if let Some(content) = &self.content {
                    for content in content {
                        write!(writer, "> ")?;
                        content.to_markdown(writer)?;
                    }
                }
                Ok(())
            }
            "heading" => {
                if let Some(attrs) = &self.attrs {
                    if let Some(level) = attrs.get("level") {
                        write!(writer, "{} ", "#".repeat(level.parse::<usize>().unwrap()))?;
                    }
                }
                if let Some(content) = &self.content {
                    for content in content {
                        content.to_markdown(writer)?;
                    }
                }
                writeln!(writer)
            }
            "orderedList" => {
                if let Some(content) = &self.content {
                    for (i, content) in content.iter().enumerate() {
                        write!(writer, "{}. ", i + 1)?;
                        content.to_markdown(writer)?;
                    }
                }
                Ok(())
            }
            "bulletList" => {
                if let Some(content) = &self.content {
                    for content in content {
                        write!(writer, "* ")?;
                        content.to_markdown(writer)?;
                    }
                }
                Ok(())
            }
            "table" => {
                if let Some(content) = &self.content {
                    let first_row_lens = content
                        .first()
                        .unwrap()
                        .content
                        .as_ref()
                        .unwrap()
                        .iter()
                        .map(|row| {
                            let mut s: String = String::new();
                            row.to_markdown(&mut s).unwrap();
                            s.len()
                        })
                        .collect::<Vec<usize>>();

                    for (i, content) in content.iter().enumerate() {
                        content.to_markdown(writer)?;
                        if i == 0 {
                            write!(writer, "|")?;
                            for len in &first_row_lens {
                                write!(writer, "{}|", "-".repeat(*len+2))?;
                            }
                            writeln!(writer)?;
                        }
                    }
                }
                Ok(())
            }
            "tableRow" => {
                if let Some(content) = &self.content {
                    let content_len = content.len();
                    write!(writer, "| ")?;
                    for (i, content) in content.iter().enumerate() {
                        content.to_markdown(writer)?;
                        if i < content_len - 1 {
                            write!(writer, " | ")?;
                        }
                    }
                    writeln!(writer, " |")?;
                }
                Ok(())
            }
            "rule" => writeln!(writer, "\n---\n"),
            "hardBreak" => writeln!(writer),
            _ => self.standard_to_markdown(writer),
        }
    }

    fn standard_to_markdown<W: std::fmt::Write>(&self, writer: &mut W) -> std::fmt::Result {
        self.marks_start(writer)?;
        if let Some(text) = &self.text {
            write!(writer, "{}", text)?;
        }
        if let Some(content) = &self.content {
            for content in content {
                content.to_markdown(writer)?;
            }
        }
        self.marks_end(writer)?;
        if !self.inline() {
            writeln!(writer)?;
        }
        Ok(())
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
                    attrs: None,
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
                        attrs: None,
                    }
                } else {
                    Content {
                        type_: "bulletList".to_string(),
                        content: Some(content),
                        text: None,
                        marks: None,
                        attrs: None,
                    }
                }
            }
            markdown::mdast::Node::InlineCode(ic) => Content {
                type_: "text".to_string(),
                content: None,
                text: Some(ic.value),
                marks: Some(vec![Mark {
                    type_: "code".to_string(),
                    attrs: None,
                }]),
                attrs: None,
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
                    attrs: None,
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
                    attrs: None,
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
                        }),
                    }]),
                    attrs: None,
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
                    attrs: None,
                }
            }
            markdown::mdast::Node::Text(t) => Content {
                type_: "text".to_string(),
                content: None,
                text: Some(t.value),
                marks: None,
                attrs: None,
            },
            markdown::mdast::Node::Code(c) => Content {
                type_: "codeBlock".to_string(),
                content: None,
                text: Some(c.value),
                marks: None,
                attrs: c
                    .lang
                    .map(|lang| vec![("language".to_string(), lang)].into_iter().collect()),
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
                    marks: None,
                    attrs: Some(
                        vec![("level".to_string(), h.depth.to_string())]
                            .into_iter()
                            .collect(),
                    ),
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
                    attrs: None,
                }
            }
            markdown::mdast::Node::ThematicBreak(_) => Content {
                type_: "rule".to_string(),
                content: None,
                text: None,
                marks: None,
                attrs: None,
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
                    attrs: None,
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
                    attrs: None,
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
                    attrs: None,
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
                    attrs: None,
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
        if self.should_flatten() {
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

    pub fn inline(&self) -> bool {
        matches!(
            self.type_.as_str(),
            "text"
                | "code"
                | "inlineCard"
                | "mention"
                | "emoji"
                | "listItem"
                | "tableCell"
                | "tableRow"
                | "table"
        )
    }

    pub fn should_flatten(&self) -> bool {
        matches!(
            self.type_.as_str(),
            "text" | "code" | "inlineCard" | "mention" | "emoji"
        ) && self.content.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_from_markdown() {
        let doc = AtlassianDoc::from_markdown("");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: Vec::new(),
            }
        );
    }

    #[test]
    fn empty_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: Vec::new(),
        };
        assert_eq!(doc.to_markdown(), "");
    }

    #[test]
    fn paragraph_from_markdown() {
        let doc = AtlassianDoc::from_markdown("Hello, world!\n");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content::paragraph("Hello, world!")],
            }
        );
    }

    #[test]
    fn paragraph_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content::paragraph("Hello, world!")],
        };
        assert_eq!(doc.to_markdown(), "Hello, world!\n");
    }

    #[test]
    fn bold_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "paragraph".to_string(),
                content: Some(vec![Content {
                    type_: "text".to_string(),
                    content: None,
                    text: Some("Hello, world!".to_string()),
                    marks: Some(vec![Mark {
                        type_: "strong".to_string(),
                        attrs: None,
                    }]),
                    attrs: None,
                }]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(doc.to_markdown(), "**Hello, world!**\n");
    }

    #[test]
    fn bold_from_markdown() {
        let doc = AtlassianDoc::from_markdown("**Hello, world!**");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "paragraph".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Hello, world!".to_string()),
                        marks: Some(vec![Mark {
                            type_: "strong".to_string(),
                            attrs: None,
                        }]),
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: None,
                }],
            }
        );
    }

    #[test]
    fn em_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "paragraph".to_string(),
                content: Some(vec![Content {
                    type_: "text".to_string(),
                    content: None,
                    text: Some("Hello, world!".to_string()),
                    marks: Some(vec![Mark {
                        type_: "em".to_string(),
                        attrs: None,
                    }]),
                    attrs: None,
                }]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(doc.to_markdown(), "*Hello, world!*\n");
    }

    #[test]
    fn em_from_markdown() {
        let doc = AtlassianDoc::from_markdown("*Hello, world!*");
        let alt_doc = AtlassianDoc::from_markdown("_Hello, world!_");
        assert_eq!(doc, alt_doc);
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "paragraph".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Hello, world!".to_string()),
                        marks: Some(vec![Mark {
                            type_: "em".to_string(),
                            attrs: None,
                        }]),
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: None,
                }],
            }
        );
    }

    #[test]
    fn strikethrough_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "paragraph".to_string(),
                content: Some(vec![Content {
                    type_: "text".to_string(),
                    content: None,
                    text: Some("Hello, world!".to_string()),
                    marks: Some(vec![Mark {
                        type_: "strike".to_string(),
                        attrs: None,
                    }]),
                    attrs: None,
                }]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(doc.to_markdown(), "~~Hello, world!~~\n");
    }

    #[test]
    fn strikethrough_from_markdown() {
        let doc = AtlassianDoc::from_markdown("~~Hello, world!~~");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "paragraph".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Hello, world!".to_string()),
                        marks: Some(vec![Mark {
                            type_: "strike".to_string(),
                            attrs: None,
                        }]),
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: None,
                }],
            }
        );
    }

    #[test]
    fn stacked_marks_from_markdown() {
        let doc = AtlassianDoc::from_markdown("***Hello**, ~~world!~~*");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "paragraph".to_string(),
                    content: Some(vec![
                        Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("Hello".to_string()),
                            marks: Some(vec![
                                Mark {
                                    type_: "strong".to_string(),
                                    attrs: None,
                                },
                                Mark {
                                    type_: "em".to_string(),
                                    attrs: None,
                                },
                            ]),
                            attrs: None,
                        },
                        Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some(", ".to_string()),
                            marks: Some(vec![Mark {
                                type_: "em".to_string(),
                                attrs: None,
                            }]),
                            attrs: None,
                        },
                        Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("world!".to_string()),
                            marks: Some(vec![
                                Mark {
                                    type_: "strike".to_string(),
                                    attrs: None,
                                },
                                Mark {
                                    type_: "em".to_string(),
                                    attrs: None,
                                },
                            ]),
                            attrs: None,
                        },
                    ]),
                    text: None,
                    marks: None,
                    attrs: None,
                }],
            }
        );
    }

    #[test]
    fn stacked_marks_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "paragraph".to_string(),
                content: Some(vec![
                    Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Hello".to_string()),
                        marks: Some(vec![
                            Mark {
                                type_: "strong".to_string(),
                                attrs: None,
                            },
                            Mark {
                                type_: "em".to_string(),
                                attrs: None,
                            },
                        ]),
                        attrs: None,
                    },
                    Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some(", ".to_string()),
                        marks: Some(vec![Mark {
                            type_: "em".to_string(),
                            attrs: None,
                        }]),
                        attrs: None,
                    },
                    Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("world!".to_string()),
                        marks: Some(vec![
                            Mark {
                                type_: "strike".to_string(),
                                attrs: None,
                            },
                            Mark {
                                type_: "em".to_string(),
                                attrs: None,
                            },
                        ]),
                        attrs: None,
                    },
                ]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(doc.to_markdown(), "***Hello****, *~~*world!*~~\n");
    }

    #[test]
    fn thematic_break_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![
                Content {
                    type_: "paragraph".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Hello".to_string()),
                        marks: None,
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: None,
                },
                Content {
                    type_: "rule".to_string(),
                    content: None,
                    text: None,
                    marks: None,
                    attrs: None,
                },
                Content {
                    type_: "paragraph".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("world!".to_string()),
                        marks: None,
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: None,
                },
            ],
        };
        assert_eq!(doc.to_markdown(), "Hello\n\n---\n\nworld!\n");
    }

    #[test]
    fn thematic_break_from_markdown() {
        let doc = AtlassianDoc::from_markdown("Hello\n\n---\n\nworld!\n");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![
                    Content {
                        type_: "paragraph".to_string(),
                        content: Some(vec![Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("Hello".to_string()),
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                    Content {
                        type_: "rule".to_string(),
                        content: None,
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                    Content {
                        type_: "paragraph".to_string(),
                        content: Some(vec![Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("world!".to_string()),
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                ],
            }
        );
    }

    #[test]
    fn code_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "paragraph".to_string(),
                content: Some(vec![Content {
                    type_: "text".to_string(),
                    content: None,
                    text: Some("Hello, world!".to_string()),
                    marks: Some(vec![Mark {
                        type_: "code".to_string(),
                        attrs: None,
                    }]),
                    attrs: None,
                }]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(doc.to_markdown(), "`Hello, world!`\n");
    }

    #[test]
    fn code_from_markdown() {
        let doc = AtlassianDoc::from_markdown("`Hello, world!`");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "paragraph".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Hello, world!".to_string()),
                        marks: Some(vec![Mark {
                            type_: "code".to_string(),
                            attrs: None,
                        }]),
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: None,
                }],
            }
        );
    }

    #[test]
    fn block_code_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "codeBlock".to_string(),
                content: None,
                text: Some("Hello, world!".to_string()),
                marks: Some(vec![Mark {
                    type_: "code".to_string(),
                    attrs: None,
                }]),
                attrs: Some(
                    vec![("language".to_string(), "rust".to_string())]
                        .into_iter()
                        .collect(),
                ),
            }],
        };
        assert_eq!(doc.to_markdown(), "```rust\nHello, world!\n```\n");
    }

    #[test]
    fn block_code_from_markdown() {
        let doc = AtlassianDoc::from_markdown("```rust\nHello, world!\n```\n");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "codeBlock".to_string(),
                    content: None,
                    text: Some("Hello, world!".to_string()),
                    marks: None,
                    attrs: Some(
                        vec![("language".to_string(), "rust".to_string())]
                            .into_iter()
                            .collect(),
                    ),
                }],
            }
        );
    }

    #[test]
    fn blockquote_from_markdown() {
        let doc = AtlassianDoc::from_markdown("> Hello, world!");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "blockquote".to_string(),
                    content: Some(vec![Content {
                        type_: "paragraph".to_string(),
                        content: Some(vec![Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("Hello, world!".to_string()),
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: None,
                }],
            }
        );
    }

    #[test]
    fn blockquote_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "blockquote".to_string(),
                content: Some(vec![Content {
                    type_: "paragraph".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Hello, world!".to_string()),
                        marks: None,
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: None,
                }]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(doc.to_markdown(), "> Hello, world!\n");
    }

    #[test]
    fn multiline_blockquote_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "blockquote".to_string(),
                content: Some(vec![
                    Content {
                        type_: "paragraph".to_string(),
                        content: Some(vec![Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("Hello, world!".to_string()),
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                    Content {
                        type_: "paragraph".to_string(),
                        content: Some(vec![Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("Hello, world!".to_string()),
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                ]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(doc.to_markdown(), "> Hello, world!\n> Hello, world!\n");
    }

    #[test]
    fn link_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "paragraph".to_string(),
                content: Some(vec![Content {
                    type_: "text".to_string(),
                    content: None,
                    text: Some("Hello, world!".to_string()),
                    marks: Some(vec![Mark {
                        type_: "link".to_string(),
                        attrs: Some(MarkAttrs {
                            href: Some("https://example.com".to_string()),
                            title: None,
                        }),
                    }]),
                    attrs: None,
                }]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(doc.to_markdown(), "[Hello, world!](https://example.com)\n");
    }

    #[test]
    fn link_from_markdown() {
        let doc = AtlassianDoc::from_markdown("[Hello, world!](https://example.com)");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "paragraph".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Hello, world!".to_string()),
                        marks: Some(vec![Mark {
                            type_: "link".to_string(),
                            attrs: Some(MarkAttrs {
                                href: Some("https://example.com".to_string()),
                                title: None,
                            }),
                        }]),
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: None,
                }],
            }
        );
    }

    #[test]
    fn link_with_title_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "paragraph".to_string(),
                content: Some(vec![Content {
                    type_: "text".to_string(),
                    content: None,
                    text: Some("Hello, world!".to_string()),
                    marks: Some(vec![Mark {
                        type_: "link".to_string(),
                        attrs: Some(MarkAttrs {
                            href: Some("https://example.com".to_string()),
                            title: Some("Example".to_string()),
                        }),
                    }]),
                    attrs: None,
                }]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(
            doc.to_markdown(),
            "[Hello, world!](https://example.com \"Example\")\n"
        );
    }

    #[test]
    fn link_with_title_from_markdown() {
        let doc = AtlassianDoc::from_markdown("[Hello, world!](https://example.com \"Example\")");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "paragraph".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Hello, world!".to_string()),
                        marks: Some(vec![Mark {
                            type_: "link".to_string(),
                            attrs: Some(MarkAttrs {
                                href: Some("https://example.com".to_string()),
                                title: Some("Example".to_string()),
                            }),
                        }]),
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: None,
                }],
            }
        );
    }

    #[test]
    fn heading_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![
                Content {
                    type_: "heading".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Heading 1".to_string()),
                        marks: None,
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: Some(
                        vec![("level".to_string(), "1".to_string())]
                            .into_iter()
                            .collect(),
                    ),
                },
                Content {
                    type_: "heading".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Heading 2".to_string()),
                        marks: None,
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: Some(
                        vec![("level".to_string(), "2".to_string())]
                            .into_iter()
                            .collect(),
                    ),
                },
                Content {
                    type_: "heading".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Heading 3".to_string()),
                        marks: None,
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: Some(
                        vec![("level".to_string(), "3".to_string())]
                            .into_iter()
                            .collect(),
                    ),
                },
                Content {
                    type_: "heading".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Heading 4".to_string()),
                        marks: None,
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: Some(
                        vec![("level".to_string(), "4".to_string())]
                            .into_iter()
                            .collect(),
                    ),
                },
                Content {
                    type_: "heading".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Heading 5".to_string()),
                        marks: None,
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: Some(
                        vec![("level".to_string(), "5".to_string())]
                            .into_iter()
                            .collect(),
                    ),
                },
                Content {
                    type_: "heading".to_string(),
                    content: Some(vec![Content {
                        type_: "text".to_string(),
                        content: None,
                        text: Some("Heading 6".to_string()),
                        marks: None,
                        attrs: None,
                    }]),
                    text: None,
                    marks: None,
                    attrs: Some(
                        vec![("level".to_string(), "6".to_string())]
                            .into_iter()
                            .collect(),
                    ),
                },
            ],
        };
        assert_eq!(
            doc.to_markdown(),
            "# Heading 1\n## Heading 2\n### Heading 3\n#### Heading 4\n##### Heading 5\n###### Heading 6\n"
        );
    }

    #[test]
    fn heading_from_markdown() {
        let doc = AtlassianDoc::from_markdown(
            "# Heading 1\n## Heading 2\n### Heading 3\n#### Heading 4\n##### Heading 5\n###### Heading 6\n",
        );
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![
                    Content {
                        type_: "heading".to_string(),
                        content: Some(vec![Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("Heading 1".to_string()),
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: Some(
                            vec![("level".to_string(), "1".to_string())]
                                .into_iter()
                                .collect(),
                        ),
                    },
                    Content {
                        type_: "heading".to_string(),
                        content: Some(vec![Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("Heading 2".to_string()),
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: Some(
                            vec![("level".to_string(), "2".to_string())]
                                .into_iter()
                                .collect(),
                        ),
                    },
                    Content {
                        type_: "heading".to_string(),
                        content: Some(vec![Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("Heading 3".to_string()),
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: Some(
                            vec![("level".to_string(), "3".to_string())]
                                .into_iter()
                                .collect(),
                        ),
                    },
                    Content {
                        type_: "heading".to_string(),
                        content: Some(vec![Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("Heading 4".to_string()),
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: Some(
                            vec![("level".to_string(), "4".to_string())]
                                .into_iter()
                                .collect(),
                        ),
                    },
                    Content {
                        type_: "heading".to_string(),
                        content: Some(vec![Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("Heading 5".to_string()),
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: Some(
                            vec![("level".to_string(), "5".to_string())]
                                .into_iter()
                                .collect(),
                        ),
                    },
                    Content {
                        type_: "heading".to_string(),
                        content: Some(vec![Content {
                            type_: "text".to_string(),
                            content: None,
                            text: Some("Heading 6".to_string()),
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: Some(
                            vec![("level".to_string(), "6".to_string())]
                                .into_iter()
                                .collect(),
                        ),
                    },
                ],
            }
        );
    }

    #[test]
    fn ordered_list_from_markdown() {
        let doc =
            AtlassianDoc::from_markdown("1. Hello, world!\n2. Hello, world!\n3. Hello, world!\n");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "orderedList".to_string(),
                    content: Some(vec![
                        Content {
                            type_: "listItem".to_string(),
                            content: Some(vec![Content {
                                type_: "paragraph".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("Hello, world!".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        },
                        Content {
                            type_: "listItem".to_string(),
                            content: Some(vec![Content {
                                type_: "paragraph".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("Hello, world!".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        },
                        Content {
                            type_: "listItem".to_string(),
                            content: Some(vec![Content {
                                type_: "paragraph".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("Hello, world!".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        },
                    ]),
                    text: None,
                    marks: None,
                    attrs: None,
                },],
            }
        );
    }

    #[test]
    fn ordered_list_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "orderedList".to_string(),
                content: Some(vec![
                    Content {
                        type_: "listItem".to_string(),
                        content: Some(vec![Content {
                            type_: "paragraph".to_string(),
                            content: Some(vec![Content {
                                type_: "text".to_string(),
                                content: None,
                                text: Some("Hello, world!".to_string()),
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                    Content {
                        type_: "listItem".to_string(),
                        content: Some(vec![Content {
                            type_: "paragraph".to_string(),
                            content: Some(vec![Content {
                                type_: "text".to_string(),
                                content: None,
                                text: Some("Hello, world!".to_string()),
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                    Content {
                        type_: "listItem".to_string(),
                        content: Some(vec![Content {
                            type_: "paragraph".to_string(),
                            content: Some(vec![Content {
                                type_: "text".to_string(),
                                content: None,
                                text: Some("Hello, world!".to_string()),
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                ]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(
            doc.to_markdown(),
            "1. Hello, world!\n2. Hello, world!\n3. Hello, world!\n"
        );
    }

    #[test]
    fn bullet_list_from_markdown() {
        let doc =
            AtlassianDoc::from_markdown("* Hello, world!\n* Hello, world!\n* Hello, world!\n");
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "bulletList".to_string(),
                    content: Some(vec![
                        Content {
                            type_: "listItem".to_string(),
                            content: Some(vec![Content {
                                type_: "paragraph".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("Hello, world!".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        },
                        Content {
                            type_: "listItem".to_string(),
                            content: Some(vec![Content {
                                type_: "paragraph".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("Hello, world!".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        },
                        Content {
                            type_: "listItem".to_string(),
                            content: Some(vec![Content {
                                type_: "paragraph".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("Hello, world!".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        },
                    ]),
                    text: None,
                    marks: None,
                    attrs: None,
                },],
            }
        );
    }

    #[test]
    fn bullet_list_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "bulletList".to_string(),
                content: Some(vec![
                    Content {
                        type_: "listItem".to_string(),
                        content: Some(vec![Content {
                            type_: "paragraph".to_string(),
                            content: Some(vec![Content {
                                type_: "text".to_string(),
                                content: None,
                                text: Some("Hello, world!".to_string()),
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                    Content {
                        type_: "listItem".to_string(),
                        content: Some(vec![Content {
                            type_: "paragraph".to_string(),
                            content: Some(vec![Content {
                                type_: "text".to_string(),
                                content: None,
                                text: Some("Hello, world!".to_string()),
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                    Content {
                        type_: "listItem".to_string(),
                        content: Some(vec![Content {
                            type_: "paragraph".to_string(),
                            content: Some(vec![Content {
                                type_: "text".to_string(),
                                content: None,
                                text: Some("Hello, world!".to_string()),
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        }]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                ]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(
            doc.to_markdown(),
            "* Hello, world!\n* Hello, world!\n* Hello, world!\n"
        );
    }

    #[test]
    fn nested_list_from_markdown() {
        let doc = AtlassianDoc::from_markdown(
            "* Hello, world!\n\t* Hello, world!\n\t* Hello, world!\n* Hello, world!\n",
        );
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "bulletList".to_string(),
                    content: Some(vec![
                        Content {
                            type_: "listItem".to_string(),
                            content: Some(vec![
                                Content {
                                    type_: "paragraph".to_string(),
                                    content: Some(vec![Content {
                                        type_: "text".to_string(),
                                        content: None,
                                        text: Some("Hello, world!".to_string()),
                                        marks: None,
                                        attrs: None,
                                    }]),
                                    text: None,
                                    marks: None,
                                    attrs: None,
                                },
                                Content {
                                    type_: "bulletList".to_string(),
                                    content: Some(vec![
                                        Content {
                                            type_: "listItem".to_string(),
                                            content: Some(vec![Content {
                                                type_: "paragraph".to_string(),
                                                content: Some(vec![Content {
                                                    type_: "text".to_string(),
                                                    content: None,
                                                    text: Some("Hello, world!".to_string()),
                                                    marks: None,
                                                    attrs: None,
                                                }]),
                                                text: None,
                                                marks: None,
                                                attrs: None,
                                            }]),
                                            text: None,
                                            marks: None,
                                            attrs: None,
                                        },
                                        Content {
                                            type_: "listItem".to_string(),
                                            content: Some(vec![Content {
                                                type_: "paragraph".to_string(),
                                                content: Some(vec![Content {
                                                    type_: "text".to_string(),
                                                    content: None,
                                                    text: Some("Hello, world!".to_string()),
                                                    marks: None,
                                                    attrs: None,
                                                }]),
                                                text: None,
                                                marks: None,
                                                attrs: None,
                                            }]),
                                            text: None,
                                            marks: None,
                                            attrs: None,
                                        },
                                    ]),
                                    text: None,
                                    marks: None,
                                    attrs: None,
                                }
                            ]),
                            text: None,
                            marks: None,
                            attrs: None,
                        },
                        Content {
                            type_: "listItem".to_string(),
                            content: Some(vec![Content {
                                type_: "paragraph".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("Hello, world!".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            }]),
                            text: None,
                            marks: None,
                            attrs: None,
                        },
                    ]),
                    text: None,
                    marks: None,
                    attrs: None,
                }]
            }
        );
    }

    #[test]
    fn table_from_markdown() {
        let doc = AtlassianDoc::from_markdown(
            "| A | B | C |\n| --- | --- | --- |\n| 1 | 2 | 3 |\n| 4 | 5 | 6 |\n",
        );
        dbg!(&doc);
        assert_eq!(
            doc,
            AtlassianDoc {
                version: 1,
                type_: "doc".to_string(),
                content: vec![Content {
                    type_: "table".to_string(),
                    content: Some(vec![
                        Content {
                            type_: "tableRow".to_string(),
                            content: Some(vec![
                                Content {
                                    type_: "tableCell".to_string(),
                                    content: Some(vec![Content {
                                        type_: "text".to_string(),
                                        content: None,
                                        text: Some("A".to_string()),
                                        marks: None,
                                        attrs: None,
                                    }]),
                                    text: None,
                                    marks: None,
                                    attrs: None,
                                },
                                Content {
                                    type_: "tableCell".to_string(),
                                    content: Some(vec![Content {
                                        type_: "text".to_string(),
                                        content: None,
                                        text: Some("B".to_string()),
                                        marks: None,
                                        attrs: None,
                                    }]),
                                    text: None,
                                    marks: None,
                                    attrs: None,
                                },
                                Content {
                                    type_: "tableCell".to_string(),
                                    content: Some(vec![Content {
                                        type_: "text".to_string(),
                                        content: None,
                                        text: Some("C".to_string()),
                                        marks: None,
                                        attrs: None,
                                    }]),
                                    text: None,
                                    marks: None,
                                    attrs: None,
                                },
                            ]),
                            text: None,
                            marks: None,
                            attrs: None,
                        },
                        Content {
                            type_: "tableRow".to_string(),
                            content: Some(vec![
                                Content {
                                    type_: "tableCell".to_string(),
                                    content: Some(vec![Content {
                                        type_: "text".to_string(),
                                        content: None,
                                        text: Some("1".to_string()),
                                        marks: None,
                                        attrs: None,
                                    }]),
                                    text: None,
                                    marks: None,
                                    attrs: None,
                                },
                                Content {
                                    type_: "tableCell".to_string(),
                                    content: Some(vec![Content {
                                        type_: "text".to_string(),
                                        content: None,
                                        text: Some("2".to_string()),
                                        marks: None,
                                        attrs: None,
                                    }]),
                                    text: None,
                                    marks: None,
                                    attrs: None,
                                },
                                Content {
                                    type_: "tableCell".to_string(),
                                    content: Some(vec![Content {
                                        type_: "text".to_string(),
                                        content: None,
                                        text: Some("3".to_string()),
                                        marks: None,
                                        attrs: None,
                                    }]),
                                    text: None,
                                    marks: None,
                                    attrs: None,
                                },
                            ]),
                            text: None,
                            marks: None,
                            attrs: None,
                        },
                        Content {
                            type_: "tableRow".to_string(),
                            content: Some(vec![
                                Content {
                                    type_: "tableCell".to_string(),
                                    content: Some(vec![Content {
                                        type_: "text".to_string(),
                                        content: None,
                                        text: Some("4".to_string()),
                                        marks: None,
                                        attrs: None,
                                    }]),
                                    text: None,
                                    marks: None,
                                    attrs: None,
                                },
                                Content {
                                    type_: "tableCell".to_string(),
                                    content: Some(vec![Content {
                                        type_: "text".to_string(),
                                        content: None,
                                        text: Some("5".to_string()),
                                        marks: None,
                                        attrs: None,
                                    }]),
                                    text: None,
                                    marks: None,
                                    attrs: None,
                                },
                                Content {
                                    type_: "tableCell".to_string(),
                                    content: Some(vec![Content {
                                        type_: "text".to_string(),
                                        content: None,
                                        text: Some("6".to_string()),
                                        marks: None,
                                        attrs: None,
                                    }]),
                                    text: None,
                                    marks: None,
                                    attrs: None,
                                },
                            ]),
                            text: None,
                            marks: None,
                            attrs: None,
                        },
                    ]),
                    text: None,
                    marks: None,
                    attrs: None,
                },],
            }
        );
    }

    #[test]
    fn table_to_markdown() {
        let doc = AtlassianDoc {
            version: 1,
            type_: "doc".to_string(),
            content: vec![Content {
                type_: "table".to_string(),
                content: Some(vec![
                    Content {
                        type_: "tableRow".to_string(),
                        content: Some(vec![
                            Content {
                                type_: "tableCell".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("A".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            },
                            Content {
                                type_: "tableCell".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("B".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            },
                            Content {
                                type_: "tableCell".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("C".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            },
                        ]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                    Content {
                        type_: "tableRow".to_string(),
                        content: Some(vec![
                            Content {
                                type_: "tableCell".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("1".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            },
                            Content {
                                type_: "tableCell".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("2".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            },
                            Content {
                                type_: "tableCell".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("3".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            },
                        ]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                    Content {
                        type_: "tableRow".to_string(),
                        content: Some(vec![
                            Content {
                                type_: "tableCell".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("4".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            },
                            Content {
                                type_: "tableCell".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("5".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            },
                            Content {
                                type_: "tableCell".to_string(),
                                content: Some(vec![Content {
                                    type_: "text".to_string(),
                                    content: None,
                                    text: Some("6".to_string()),
                                    marks: None,
                                    attrs: None,
                                }]),
                                text: None,
                                marks: None,
                                attrs: None,
                            },
                        ]),
                        text: None,
                        marks: None,
                        attrs: None,
                    },
                ]),
                text: None,
                marks: None,
                attrs: None,
            }],
        };
        assert_eq!(
            doc.to_markdown(),
            "| A | B | C |\n|---|---|---|\n| 1 | 2 | 3 |\n| 4 | 5 | 6 |\n"
        );
    }
}
