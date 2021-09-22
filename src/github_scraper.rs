/// Searches for discussions on GitHub marked with "opportunity"

use regex::Regex;
use lazy_static::lazy_static;
use std::collections::BTreeSet;

// When production-ready, replace with "/UWAppDev/community/discussions"
macro_rules! DISCUSSIONS_BASE_URL { () => { "UWAppDev/opportunities-forwarding-bot/discussions/" }; }
macro_rules! OPPORTUNITIES_POST_TO_URL { () => { concat!("https://github.com/", DISCUSSIONS_BASE_URL!(), "categories/opportunities/") }; }
macro_rules! DISCUSSION_LINK_REGEX {
    () => { concat!(r"/", DISCUSSIONS_BASE_URL!(), r"[/]*(?P<id>\d+)"); };
}

/// Where _users_ should post new opportunities.
pub const OPPORTUNITIES_POST_TO_URL: &'static str = OPPORTUNITIES_POST_TO_URL!();

pub struct DiscussionLink {
    content: String,
    id: u16,
}

pub struct DiscussionPost {
    content: String,
    url: DiscussionLink,
}

impl DiscussionLink {
    pub fn new(full_link_text: String, id: u16) -> DiscussionLink {
        DiscussionLink {
            content: full_link_text,
            id
        }
    }

    /// Pull and return all links to discussion posts from [text].
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
}


#[cfg(test)]
mod tests {
    use super::DiscussionLink;

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
        let source = include_str!("../res/ghub_opportunities_list_snapshot.html");
        let links = DiscussionLink::pull_from(&source);

        assert_eq!(links.len(), 4, "Ensure we find three links in our source. Three discussions links and one 'welcome' link.");
        assert_eq!(links[1].get_id(), 3);
        assert_eq!(links[2].get_id(), 5);
    }
}

