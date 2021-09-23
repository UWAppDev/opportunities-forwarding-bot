//! Utility functions for walking through a parsed HTML tree.

use select::node::Node;
use select::document::Document;
use select::predicate::*;

pub struct MarkdownWalker {
    buffer: Vec<String>,
}

impl MarkdownWalker {
    /// Get an empty [MarkdownWalker].
    /// This walker can then walk the DOM via [walk].
    pub fn new() -> MarkdownWalker {
        MarkdownWalker {
            buffer: Vec::new()
        }
    }

    /// Walk to visit all of `node`'s children.
    fn visit_children(&mut self, node: &Node) {
        for child in node.children() {
            self.visit(&child);
        }
    }

    /// Adds the given str to the output.
    fn add<T>(&mut self, text: T) where String: From<T> {
        self.buffer.push(String::from(text));
    }

    /// Walk the DOM.
    fn visit(&mut self, node: &Node) {
        if node.is(Text) {
            // We're a text node. Just output our text.
            self.add(node.text());
        }
        else if node.is(Comment) {
            // Skip comments.
            return;
        }
        else if node.is(Name("p")) || node.is(Name("div")) || node.is(Name("tr")) {
            self.visit_children(node);

            // Paragraphs have a trailing newline
            self.add("\n");
        }
        else if node.is(Name("br")) {
            // <br/>s don't have content.
            self.add("\n");
        }
        else if node.is(Name("a")) {
            // A link
            let target = node.attr("href");

            self.add("[");
            self.visit_children(node);
            self.add("]");

            if let Some(target) = target {
                self.add("(");
                self.add(target);
                self.add(") ");
            }
        }
        else if node.is(Name("i")) || node.is(Name("emph")) {
            self.add("_");
            self.visit_children(node);
            self.add("_");
        }
        else if node.is(Name("b")) || node.is(Name("strong")) {
            self.add("**");
            self.visit_children(node);
            self.add("**");
        }
        else if node.is(Name("code")) {
            self.add("`");
            self.visit_children(node);
            self.add("`");
        }
        else if node.is(Name("pre")) {
            self.add("\n```\n");
            self.visit_children(node);
            self.add("\n```\n");
        }
        else if node.is(Name("h1")) {
            self.add("\n# ");
            self.visit_children(node);
            self.add("\n");
        }
        else if node.is(Name("h2")) {
            self.add("\n## ");
            self.visit_children(node);
            self.add("\n");
        }
        else if node.is(Name("h3")) {
            self.add("\n### ");
            self.visit_children(node);
            self.add("\n");
        }
        else if node.is(Name("quote")) {
            self.add("\n> ");
            self.visit_children(node);
            self.add("\n");
        }
        else {
            self.visit_children(node);
        }
    }

    /// Parse the given HTML and walk its tree.
    pub fn start(&mut self, html: &str) {
        let html = format!("<html>{}</html>", html);
        let doc = Document::from(&html[..]);

        if let Some(node) = doc.nth(0) {
            self.visit(&node);
        }
    }

    /// Get the content this has accumulated.
    pub fn get_content(&self) -> String {
        let joined = self.buffer.join("");

        joined.trim().to_string()
    }
}

/// Walks the given `html` using a [MarkdownWalker]
/// and returns the collected content.
pub fn html_to_md(html: &str) -> String {
    let mut walker = MarkdownWalker::new();
    walker.start(&html[..]);
    walker.get_content()
}

#[cfg(test)]
mod tests {
    use super::MarkdownWalker;

    #[test]
    fn test_simple_html2md() {
        let html = r#"
<div>
<h1>This is a test</h1>
<p>Of <i>a</i> thing.</p>
</div>
        "#;
        let md = r#"# This is a test

Of _a_ thing."#;
        let mut walker = MarkdownWalker::new();
        walker.start(&html);

        assert_eq!(walker.get_content(), md);
    }
}

