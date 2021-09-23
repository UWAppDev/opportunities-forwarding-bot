/// Searches for discussions on GitHub marked with "opportunity"

use regex::Regex;
use lazy_static::lazy_static;
use std::collections::BTreeSet;

use select::document::Document;
use select::predicate::{Class, Attr};

// When production-ready, replace with "/UWAppDev/community/discussions"
macro_rules! DISCUSSIONS_BASE_URL { () => { "UWAppDev/opportunities-forwarding-bot/discussions/" }; }
macro_rules! OPPORTUNITIES_LIST_URL { () => { concat!("https://github.com/", DISCUSSIONS_BASE_URL!(), "categories/opportunities/") }; }
macro_rules! DISCUSSION_LINK_REGEX {
    () => { concat!(r"/", DISCUSSIONS_BASE_URL!(), r"[/]*(?P<id>\d+)"); };
}

/// Where _users_ should post new opportunities.
pub const OPPORTUNITIES_POST_TO_URL: &'static str = OPPORTUNITIES_LIST_URL!();

#[derive(Clone, Debug)]
pub struct DiscussionLink {
    content: String,
    id: u16,
}

#[derive(Clone, Debug)]
pub struct DiscussionPost {
    content: String,
    author: String,
    url: DiscussionLink,
}

#[derive(Debug)]
struct PostNotFoundError;

impl DiscussionLink {
    fn new(full_link_text: String, id: u16) -> DiscussionLink {
        DiscussionLink {
            content: full_link_text,
            id
        }
    }

    /// Extract all links to discussion posts from this' remote repository.
    pub async fn fetch() -> Result<Vec<DiscussionLink>, Box<dyn std::error::Error>> {
        let html = reqwest::get(OPPORTUNITIES_LIST_URL!()).await?.text().await?;

        Ok(Self::pull_from(&html))
    }


    /// Pull and return all links to discussion posts from `text`.
    pub fn pull_from(text: &str) -> Vec<DiscussionLink> {
        lazy_static! {
            static ref RE: Regex = Regex::new(DISCUSSION_LINK_REGEX!()).unwrap();
        }
        let mut seen_ids: BTreeSet<u16> = BTreeSet::new();

        let mut res: Vec<DiscussionLink> =
            RE.captures_iter(text)
                .map(|captures| {
                    let full_link: String = captures[0].into();
                    let id: u16 = captures["id"].parse().unwrap();

                    DiscussionLink::new(full_link, id)
                })
                .filter(|link| {
                    if seen_ids.contains(&link.get_id()) {
                        return false;
                    }

                    seen_ids.insert(link.get_id());
                    true
                })
                .collect();
        res.sort_by(|a, b| a.id.cmp(&b.id));

        res
    }

    /// Get the id associated with the link.
    pub fn get_id(&self) -> u16 {
        self.id
    }

    /// Get the full URL (including `https://`) to this' target.
    ///
    /// For example:
    /// ```
    /// let link = DiscussionLink::new("/UWAppDev/community/discussions/0", 0);
    /// assert_eq!(link.get_url(), "https://github.com/UWAppDev/community/discussions/0");
    /// ```
    ///
    pub fn get_url(&self) -> String {
        if self.content.starts_with("http") {
            self.content.clone()
        } else if self.content.starts_with("github") || self.content.starts_with("www.") {
            format!("https://{}", self.content)
        } else if self.content.starts_with("/") {
            format!("https://github.com{}", self.content)
        } else {
            format!("https://github.com/{}", self.content)
        }
    }
}

impl std::fmt::Display for PostNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "No discussion post found in the document associated with the link")
    }
}

impl std::error::Error for PostNotFoundError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl DiscussionPost {
    /// Creates a new discussion post with `content` and location `link`.
    /// This does not fetch or verify the `content` using `link`.
    fn new(content: String, author: String, link: DiscussionLink) -> DiscussionPost {
        DiscussionPost {
            content,
            author,
            url: link
        }
    }

    /// Fetches all applicable discussion posts from this project's GitHub.
    /// As this involves network communication, errors are possible.
    pub async fn fetch_from(link: DiscussionLink) -> Result<DiscussionPost, Box<dyn std::error::Error>> {
        let html = reqwest::get(link.get_url()).await?.text().await?;

        Self::pull_from(link, &html)
    }

    /// Create a DiscussionPost from given `html` that has been fetched from `link`.
    fn pull_from(link: DiscussionLink, html: &str) -> Result<DiscussionPost, Box<dyn std::error::Error>> {
        let document = Document::from(html);
        let first_comment = document
                .find(Class("unminimized-comment"))
                .next();

        if None == first_comment {
            return Err(Box::new(PostNotFoundError));
        }
        let first_comment = first_comment.unwrap();

        let author = first_comment.find(select::predicate::And(Class("author"), select::predicate::Name("a")))
                .next();
        let content = first_comment.find(select::predicate::And(Attr("data-paste-markdown-skip", ""), Class("js-translation-source")))
                .next();

        let author = match author {
            Some(node) => node.text(),
            None => "Unknown Author".to_string(),
        };

        let content = match content {
            Some(node) => node.text(), // TODO: Walk the HTML tree here to convert it to markdown.
            None => "Unable to find content for this post!!!".to_string(),
        };

        let author = author.trim().to_string();
        let content = content.trim().to_string();

        Ok(DiscussionPost::new(content, author, link))
    }

    /// Get the markdown content of this post.
    pub fn get_content(&self) -> &str {
        &self.content[..]
    }

    /// Get the publicly-shown name of the author of this post.
    pub fn get_author(&self) -> &str {
        &self.author[..]
    }

    /// Gets the [DiscussionLink] that points to this' content.
    pub fn get_link(&self) -> &DiscussionLink {
        &self.url
    }
}

#[cfg(test)]
mod tests {
    use super::{ DiscussionLink, DiscussionPost };

    #[test]
    fn test_link_scrape_simple() {
        let source = format!("/{}123, /{}/0", DISCUSSIONS_BASE_URL!(), DISCUSSIONS_BASE_URL!());
        let links = DiscussionLink::pull_from(&source);

        assert_eq!(links.len(), 2, "Ensure we find two links in {}", source);
        assert_eq!(links[1].get_id(), 123);
        assert_eq!(links[1].id, 123);
        assert_eq!(links[0].id, 0);
        assert_eq!(links[1].content, format!("/{}123", DISCUSSIONS_BASE_URL!()));
    }

    #[test]
    fn test_link_scrape_github() {
        let source = include_str!("../res/tests/ghub_opportunities_list_snapshot.html");
        let links = DiscussionLink::pull_from(&source);

        assert_eq!(links.len(), 4, "Ensure we find three links in our source. Three discussions links and one 'welcome' link.");
        assert_eq!(links[1].get_id(), 3);
        assert_eq!(links[2].get_id(), 5);
    }

    #[test]
    fn test_complete_short_link() {
        let link = DiscussionLink::new("/foo/bar".to_string(), 0);
        assert_eq!(link.get_url(), "https://github.com/foo/bar");
    }

    #[test]
    fn test_complete_full_link() {
        let link = DiscussionLink::new("https://github.com/a/test".to_string(), 1);
        assert_eq!(link.get_url(), "https://github.com/a/test");
    }

    #[test]
    fn test_discussion_post_from_html() {
        let link = DiscussionLink::new("https://github.com/UWAppDev/opportunities-forwarding-bot/discussions/5".to_string(), 5);
        let post = DiscussionPost::pull_from(link, include_str!("../res/tests/ghub_opportunities_post_snapshot.html")).unwrap();
        assert_eq!(post.get_author(), "personalizedrefrigerator");
        assert_eq!(post.get_content(), "This is an opportunity to test the opportunities-forwarding-bot!");
    }

    // tokio::test because we're doing a test of an async function
    #[tokio::test]
    async fn test_discussion_post_fetch_from_internet() {
        let link = DiscussionLink::new("https://github.com/UWAppDev/opportunities-forwarding-bot/discussions/3".to_string(), 3);
        let post = DiscussionPost::fetch_from(link).await.expect("Unable to fetch remote discussion post!");
        assert_eq!(post.get_author(), "personalizedrefrigerator");
        assert!(post.get_content().contains("This post will be used to test the bot!"), "{} does not contian the test string!", post.get_content());
    }
}

