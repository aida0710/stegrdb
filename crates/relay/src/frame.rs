use crate::RelayError;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Ethernetヘッダと、Linuxの通常の受信バッファで扱うフレーム長の上限。
pub const MIN_FRAME_SIZE: usize = 14;
pub const MAX_FRAME_SIZE: usize = 65_535;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub id: Uuid,
    pub bytes: Bytes,
}

impl Frame {
    pub fn new(bytes: Bytes) -> Result<Self, RelayError> {
        let frame = Self { id: Uuid::new_v4(), bytes };
        frame.validate()?;
        Ok(frame)
    }

    pub fn validate(&self) -> Result<(), RelayError> {
        if !(MIN_FRAME_SIZE..=MAX_FRAME_SIZE).contains(&self.bytes.len()) {
            return Err(RelayError::permanent("Ethernetフレームの長さが範囲外です"));
        }
        Ok(())
    }
}

/// コアは値を解釈せず、発行したプラグインにそのまま返す。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Receipt(pub String);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Delivery {
    pub frame: Frame,
    pub receipt: Receipt,
}
