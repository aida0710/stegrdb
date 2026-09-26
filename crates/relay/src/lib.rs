//! 中継プラグインとの境界。DB、ネットワークデバイス、フィルタの型を持ち込まない。
mod error;
mod frame;
mod registry;

pub use error::RelayError;
pub use frame::{Delivery, Frame, Receipt, MAX_FRAME_SIZE, MIN_FRAME_SIZE};
pub use registry::PluginRegistry;

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct RelayContext {
    pub node_id: String,
    pub channel: String,
}

impl RelayContext {
    pub fn validate(&self) -> Result<(), RelayError> {
        // DB以外の実装でも識別子のメモリ使用量を一定に抑える。
        const MAX_IDENTIFIER_LENGTH: usize = 128;
        for (name, value) in [("node_id", &self.node_id), ("channel", &self.channel)] {
            if value.trim().is_empty() || value.len() > MAX_IDENTIFIER_LENGTH || value.chars().any(char::is_control) {
                return Err(RelayError::permanent(format!("{name}は1〜{MAX_IDENTIFIER_LENGTH}バイトの識別子で指定してください")));
            }
        }
        Ok(())
    }
}

#[async_trait]
pub trait Relay: Send + Sync {
    /// 成功は中継先が受け付けたことを表す。対向NICへの到達保証ではない。
    /// 同一Frame.idによる再試行を冪等に扱う。失敗時の部分成功も許容する。
    async fn publish(&self, frames: &[Frame]) -> Result<(), RelayError>;

    /// 非破壊取得。acknowledgeされるまで再取得可能で、自ノード由来を返さない。
    /// 順序・永続性・保持期間は各プラグインが文書化する。
    async fn receive(&self, limit: usize) -> Result<Vec<Delivery>, RelayError>;

    /// NICへの送信成功、または明示的なフィルタ拒否の後に呼ぶ。再試行は冪等。
    async fn acknowledge(&self, receipts: &[Receipt]) -> Result<(), RelayError>;
}

#[async_trait]
pub trait RelayPlugin: Send + Sync {
    fn name(&self) -> &'static str;

    /// 同じchannel/node_idの同時利用を拒否する。Dropでノードの占有を解除する。
    async fn connect(&self, context: RelayContext, options: Value) -> Result<Arc<dyn Relay>, RelayError>;
}
