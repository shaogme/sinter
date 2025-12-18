use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Tag};
use sinter_core::ContentNode;

/// State machine for transforming Markdown events into an AST.
/// Uses a pushdown automaton (stack-based state machine) to handle nested structures.
pub struct AstStateMachine {
    stack: Vec<Frame>,
}

struct Frame {
    tag: Option<FrameType>,
    children: Vec<ContentNode>,
}

enum FrameType {
    Container(fn(Vec<ContentNode>) -> ContentNode),
    Heading(u8, Option<String>, Vec<String>),
    List(bool),
    Link(String, Option<String>),
    Image(String, Option<String>),
    CodeBlock(Option<String>),
}

impl AstStateMachine {
    pub fn new() -> Self {
        Self {
            stack: vec![Frame {
                tag: None,
                children: Vec::new(),
            }],
        }
    }

    pub fn consume(&mut self, event: Event) {
        match event {
            Event::Start(tag) => self.enter_node(tag),
            Event::End(_) => self.exit_node(),
            Event::Text(text) => self.append_text(text),
            Event::Code(text) => self.append_inline_code(text),
            Event::SoftBreak => self.append_text(CowStr::from(" ")),
            Event::HardBreak => self.append_text(CowStr::from("\n")),
            Event::Rule => self.append_node(ContentNode::ThematicBreak),
            Event::Html(text) | Event::InlineHtml(text) => self.append_node(ContentNode::Html {
                value: text.to_string(),
            }),
            Event::InlineMath(text) => self.append_node(ContentNode::Math {
                value: text.to_string(),
                display: false,
            }),
            Event::DisplayMath(text) => self.append_node(ContentNode::Math {
                value: text.to_string(),
                display: true,
            }),
            Event::TaskListMarker(checked) => {
                self.append_node(ContentNode::TaskListMarker { checked })
            }
            // TODO: Add support for Footnotes if needed
            _ => {}
        }
    }

    pub fn finish(mut self) -> Vec<ContentNode> {
        // Gracefully close any unclosed tags (though parser usually guarantees structure)
        while self.stack.len() > 1 {
            self.exit_node();
        }
        self.stack.pop().unwrap().children
    }

    fn enter_node(&mut self, tag: Tag) {
        let frame_type = match tag {
            Tag::Paragraph => Some(FrameType::Container(|c| ContentNode::Paragraph {
                children: c,
            })),
            Tag::Heading {
                level, id, classes, ..
            } => {
                let l = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
                Some(FrameType::Heading(
                    l,
                    id.map(|s| s.to_string()),
                    classes.into_iter().map(|s| s.to_string()).collect(),
                ))
            }
            Tag::BlockQuote(_) => Some(FrameType::Container(|c| ContentNode::BlockQuote {
                children: c,
            })),
            Tag::CodeBlock(kind) => {
                let lang = match kind {
                    CodeBlockKind::Fenced(l) => Some(l.to_string()),
                    CodeBlockKind::Indented => None,
                };
                Some(FrameType::CodeBlock(lang))
            }
            Tag::List(ordered) => Some(FrameType::List(ordered.is_some())),
            Tag::Item => Some(FrameType::Container(|c| ContentNode::ListItem {
                children: c,
            })),
            Tag::Emphasis => Some(FrameType::Container(|c| ContentNode::Emphasis {
                children: c,
            })),
            Tag::Strong => Some(FrameType::Container(|c| ContentNode::Strong {
                children: c,
            })),
            Tag::Strikethrough => Some(FrameType::Container(|c| ContentNode::Strikethrough {
                children: c,
            })),
            Tag::Link {
                dest_url, title, ..
            } => {
                let title_s = title.to_string();
                let title_opt = if title_s.is_empty() {
                    None
                } else {
                    Some(title_s)
                };
                Some(FrameType::Link(dest_url.to_string(), title_opt))
            }
            Tag::Image {
                dest_url, title, ..
            } => {
                let title_s = title.to_string();
                let title_opt = if title_s.is_empty() {
                    None
                } else {
                    Some(title_s)
                };
                Some(FrameType::Image(dest_url.to_string(), title_opt))
            }
            Tag::Table(_) => Some(FrameType::Container(|c| ContentNode::Table { children: c })),
            Tag::TableHead => Some(FrameType::Container(|c| ContentNode::TableHead {
                children: c,
            })),
            Tag::TableRow => Some(FrameType::Container(|c| ContentNode::TableRow {
                children: c,
            })),
            Tag::TableCell => Some(FrameType::Container(|c| ContentNode::TableCell {
                children: c,
            })),
            _ => None,
        };

        if let Some(ft) = frame_type {
            self.stack.push(Frame {
                tag: Some(ft),
                children: Vec::new(),
            });
        }
    }

    fn exit_node(&mut self) {
        if self.stack.len() > 1 {
            let frame = self.stack.pop().unwrap();

            // Transform the completed frame into a ContentNode
            let node = match frame.tag {
                Some(FrameType::Container(builder)) => builder(frame.children),
                Some(FrameType::Heading(level, id, classes)) => ContentNode::Heading {
                    level,
                    id,
                    classes,
                    children: frame.children,
                },
                Some(FrameType::List(ordered)) => ContentNode::List {
                    ordered,
                    children: frame.children,
                },
                Some(FrameType::Link(url, title)) => ContentNode::Link {
                    url,
                    title,
                    children: frame.children,
                },
                Some(FrameType::Image(url, title)) => {
                    let alt = frame
                        .children
                        .iter()
                        .map(|c| match c {
                            ContentNode::Text { value } => value.as_str(),
                            _ => "",
                        })
                        .collect::<String>();
                    ContentNode::Image { url, title, alt }
                }
                Some(FrameType::CodeBlock(lang)) => {
                    let code = frame
                        .children
                        .iter()
                        .map(|c| match c {
                            ContentNode::Text { value } => value.as_str(),
                            _ => "",
                        })
                        .collect::<String>();
                    ContentNode::CodeBlock { lang, code }
                }
                None => unreachable!("Root frame should not be popped via exit_node"),
            };

            self.append_node(node);
        }
    }

    fn append_text(&mut self, text: CowStr) {
        if let Some(top) = self.stack.last_mut() {
            // Optimization: Merge adjacent text nodes?
            // For now, simple append
            top.children.push(ContentNode::Text {
                value: text.to_string(),
            });
        }
    }

    fn append_inline_code(&mut self, text: CowStr) {
        if let Some(top) = self.stack.last_mut() {
            // Currently generic Text, but could be specific InlineCode node in future
            top.children.push(ContentNode::Text {
                value: text.to_string(),
            });
        }
    }

    fn append_node(&mut self, node: ContentNode) {
        if let Some(parent) = self.stack.last_mut() {
            parent.children.push(node);
        }
    }
}

/// Convenience function to parse
pub fn parse(parser: pulldown_cmark::Parser) -> Vec<ContentNode> {
    let mut machine = AstStateMachine::new();
    for event in parser {
        machine.consume(event);
    }
    machine.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pulldown_cmark::{Options, Parser};

    fn parse_md(md: &str) -> Vec<ContentNode> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(Options::ENABLE_MATH);
        let parser = Parser::new_ext(md, options);
        parse(parser)
    }

    #[test]
    fn test_basic_structure() {
        let ast = parse_md("# Title\n\nParagraph text.");
        assert_eq!(ast.len(), 2);

        match &ast[0] {
            ContentNode::Heading { level, .. } => {
                assert_eq!(*level, 1);
            }
            _ => panic!("Expected Heading"),
        }

        match &ast[1] {
            ContentNode::Paragraph { children } => match &children[0] {
                ContentNode::Text { value } => assert_eq!(value, "Paragraph text."),
                _ => panic!("Expected Text"),
            },
            _ => panic!("Expected Paragraph"),
        }
    }

    #[test]
    fn test_heading_attributes() {
        // Requires ENABLE_HEADING_ATTRIBUTES
        let md = "# Title { #myid .myclass }";
        let ast = parse_md(md);
        match &ast[0] {
            ContentNode::Heading { id, classes, .. } => {
                assert_eq!(id.as_deref(), Some("myid"));
                assert_eq!(classes[0], "myclass");
            }
            _ => panic!("Expected Heading"),
        }
    }

    #[test]
    fn test_math() {
        let md = "$E=mc^2$";
        let ast = parse_md(md);
        match &ast[0] {
            ContentNode::Paragraph { children } => match &children[0] {
                ContentNode::Math { value, display } => {
                    assert_eq!(value, "E=mc^2");
                    assert!(!display);
                }
                _ => panic!("Expected Math"),
            },
            _ => panic!("Expected Paragraph enum {:#?}", ast[0]),
        }
    }
}
