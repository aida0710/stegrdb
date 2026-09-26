use crate::{Relay, RelayContext, RelayError, RelayPlugin};
use serde_json::Value;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Default)]
pub struct PluginRegistry {
    plugins: BTreeMap<&'static str, Box<dyn RelayPlugin>>,
}

impl PluginRegistry {
    pub fn register(&mut self, plugin: impl RelayPlugin + 'static) -> Result<(), RelayError> {
        if self.plugins.contains_key(plugin.name()) {
            return Err(RelayError::permanent(format!("プラグインが重複しています: {}", plugin.name())));
        }
        self.plugins.insert(plugin.name(), Box::new(plugin));
        Ok(())
    }

    pub fn names(&self) -> Vec<&'static str> {
        self.plugins.keys().copied().collect()
    }

    pub async fn connect(&self, name: &str, context: RelayContext, options: Value) -> Result<Arc<dyn Relay>, RelayError> {
        context.validate()?;
        let plugin = self.plugins.get(name).ok_or_else(|| RelayError::permanent(format!("未登録の中継プラグイン: {name}")))?;
        plugin.connect(context, options).await
    }
}
