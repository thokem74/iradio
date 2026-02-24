use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteItem {
    pub label: String,
    pub action: String,
}

pub fn fuzzy_filter(items: &[PaletteItem], query: &str) -> Vec<PaletteItem> {
    if query.trim().is_empty() {
        return items.to_vec();
    }

    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(query, CaseMatching::Ignore, Normalization::Smart);

    let mut scored = Vec::new();
    for item in items {
        let score = pattern.score(item.label.as_str(), &mut matcher);
        if let Some(score) = score {
            scored.push((score, item.clone()));
        }
    }

    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.label.cmp(&b.1.label)));
    scored.into_iter().map(|(_, item)| item).collect()
}
