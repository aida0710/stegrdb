#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::LinuxSocket;

use async_trait::async_trait;
use std::io;

#[async_trait]
pub trait PacketIo: Send + Sync {
    /// 1フレームを読み取る。成功時の長さはbufferの範囲内でなければならない。
    async fn receive(&self, buffer: &mut [u8]) -> io::Result<usize>;
    /// 成功はカーネルへの引き渡しを表し、対向マシンへの到達保証ではない。
    async fn send(&self, frame: &[u8]) -> io::Result<()>;
}
