pub(super) fn rewrite_candidate<F>(candidate: &str, build: &F) -> String
where
    F: Fn(&str) -> Option<String>,
{
    let mut parts = candidate.split_whitespace();
    let Some(url) = parts.next() else {
        return String::new();
    };
    let mut rewritten = build(url).unwrap_or_else(|| url.to_owned());
    for descriptor in parts {
        rewritten.push(' ');
        rewritten.push_str(descriptor);
    }
    rewritten
}

#[cfg(test)]
mod tests {
    use super::rewrite_candidate;

    #[test]
    fn rewritten_candidates_own_urls_and_preserve_descriptors() {
        for _ in 0..10_000 {
            assert_eq!(
                rewrite_candidate(" /photo.png  640w ", &|url| Some(format!(
                    "/proxy?url={url}"
                ))),
                "/proxy?url=/photo.png 640w"
            );
        }
        assert_eq!(rewrite_candidate("photo.png 2x", &|_| None), "photo.png 2x");
        assert_eq!(rewrite_candidate("  ", &|_| None), "");
    }
}
