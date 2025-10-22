use crate::{
    builder::Builder,
    client::ClientImplementation,
    formatter::{self, FormatItem, FormatModel},
    name_matcher::matcher,
};

use rustemon::model::items::Item;

pub struct ItemCommand<'a> {
    builder: &'a mut Builder,
    client: &'a dyn ClientImplementation,
    item_name: String,
}

impl ItemCommand<'_> {
    pub async fn execute(client: &dyn ClientImplementation, item_name: String) -> Builder {
        let mut builder = Builder::default();

        ItemCommand {
            builder: &mut builder,
            client,
            item_name,
        }
        ._execute()
        .await;

        builder
    }

    async fn _execute(&mut self) {
        let item = match self.fetch_item().await {
            Ok(item) => item,
            Err(error_message) => {
                self.builder.append(error_message);
                return;
            }
        };

        self.builder.append(formatter::white("Item"));
        self.builder.append('\n');
        self.builder.append(FormatItem::new(item).format());
    }

    async fn fetch_item(&self) -> Result<Item, String> {
        let successful_match =
            match matcher::match_name(&self.item_name, matcher::MatcherType::Item) {
                Ok(successful_match) => Ok(successful_match),
                Err(no_match) => Err(no_match.0),
            }?;

        let result = self
            .client
            .fetch_item(&successful_match.suggested_name)
            .await;

        match result {
            Ok(item) => Ok(item),
            Err(_) => {
                let output = matcher::build_unknown_name(
                    &successful_match.keyword,
                    &successful_match.suggested_name,
                );
                Err(output)
            }
        }
    }
}
