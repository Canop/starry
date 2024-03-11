use {
    crate::*,
    minimad::{
        OwningTemplateExpander,
        TextTemplate,
    },
    termimad::*,
};

static TEMPLATE: &str = r#"
${change-count} changes
${cropped
${kept-count} most significant ones:
}
|:-:|:-:|:-:|
|**owner**|**name**|**last**|**trend**|**now**|**url** (ctrl-click to open)|
|-:|:-|-:|:-:|-:|:-|
${changes
|${owner}|**${name}**|${last}|${trend}|**${now}|${url}|
}
|-|-|-|-|-|-|
"#;

pub struct ChangeReport<'c> {
    changes: &'c [RepoChange],
    max_rows: usize,
}

impl<'c> ChangeReport<'c> {
    pub fn new(
        changes: &'c [RepoChange],
        max_rows: usize,
    ) -> Self {
        Self { changes, max_rows }
    }
    pub fn print(
        &self,
        skin: &MadSkin,
    ) {
        if self.changes.is_empty() {
            println!("no change");
            return;
        }
        let mut expander = OwningTemplateExpander::new();
        expander
            .set_default("")
            .set("change-count", self.changes.len());
        for change in self.changes.iter().take(self.max_rows) {
            expander
                .sub("changes")
                .set("owner", &change.repo_id.owner)
                .set("name", &change.repo_id.name)
                .set(
                    "last",
                    change.old_stars.map_or("".to_string(), |s| s.to_string()),
                )
                //.set_md("trend", format!("{} {}", change.value(), change.trend_markdown()))
                .set_md("trend", change.trend_markdown())
                .set("now", change.new_stars)
                .set("url", change.url());
        }
        if self.changes.len() > self.max_rows {
            expander.sub("cropped").set("kept-count", self.max_rows);
        }
        let template = TextTemplate::from(TEMPLATE);
        let text = expander.expand(&template);
        let (width, _) = terminal_size();
        let fmt_text = FmtText::from_text(skin, text, Some(width as usize));
        print!("{}", fmt_text);
    }
}
