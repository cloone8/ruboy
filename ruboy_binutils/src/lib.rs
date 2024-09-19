use std::fmt::Display;
use unicode_width::UnicodeWidthStr;

pub mod cli;

#[derive(Default)]
pub struct ListOutput {
    items: Vec<ListItem>,
}

impl ListOutput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, item: ListItem) {
        self.items.push(item)
    }

    pub fn add_single(&mut self, label: impl ToString, value: impl ToString) {
        self.add(ListItem::Single {
            label: label.to_string(),
            value: value.to_string(),
        })
    }

    pub fn add_multiple(&mut self, label: impl ToString, values: Vec<impl ToString>) {
        self.add(ListItem::Multiple {
            label: label.to_string(),
            values: values.into_iter().map(|v| v.to_string()).collect(),
        })
    }
}

impl Display for ListOutput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label_width = self
            .items
            .iter()
            .map(|item| match item {
                ListItem::Single { label, .. } => label.width(),
                ListItem::Multiple { label, .. } => label.width(),
            })
            .max()
            .unwrap_or(0);

        for item in &self.items {
            writeln!(f, "{}", item.output(label_width))?;
        }

        Ok(())
    }
}

pub enum ListItem {
    Single { label: String, value: String },
    Multiple { label: String, values: Vec<String> },
}

impl ListItem {
    pub fn output(&self, label_width: usize) -> String {
        let label = match self {
            ListItem::Single { label, .. } => label,
            ListItem::Multiple { label, .. } => label,
        };

        assert!(label.width() <= label_width, "Label too long");

        let padding = " ".repeat(label_width - label.width());

        let padded_label = if !label.is_empty() {
            format!("{}: {}", label, padding)
        } else {
            "".to_string()
        };

        match self {
            ListItem::Single { label: _, value } => {
                format!("{}{}", padded_label, value)
            }
            ListItem::Multiple { label: _, values } => {
                let mut result = String::new();

                for value in values {
                    result.push_str(format!("    - {}\n", value).as_str());
                }

                format!("{}\n{}", padded_label, result)
            }
        }
    }
}
