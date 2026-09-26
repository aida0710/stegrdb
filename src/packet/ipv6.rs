use super::{transport, PacketError, PacketMetadata};
use std::net::{IpAddr, Ipv6Addr};

const HEADER_LENGTH: usize = 40;
// 不正な拡張ヘッダ連鎖で解析を長時間占有させない。
const MAX_EXTENSION_HEADERS: usize = 8;

pub(super) fn parse(payload: &[u8], packet: &mut PacketMetadata) -> Result<(), PacketError> {
    if payload.len() < HEADER_LENGTH || payload[0] >> 4 != 6 {
        return Err(PacketError::Malformed);
    }
    let payload_length = usize::from(u16::from_be_bytes([payload[4], payload[5]]));
    let total_length = HEADER_LENGTH + payload_length;
    if total_length > payload.len() {
        return Err(PacketError::Malformed);
    }
    packet.src_ip = Some(IpAddr::V6(Ipv6Addr::from(<[u8; 16]>::try_from(&payload[8..24]).map_err(|_| PacketError::Malformed)?)));
    packet.dst_ip = Some(IpAddr::V6(Ipv6Addr::from(<[u8; 16]>::try_from(&payload[24..40]).map_err(|_| PacketError::Malformed)?)));
    let mut protocol = payload[6];
    let mut remaining = &payload[HEADER_LENGTH..total_length];
    let mut extension_count = 0;
    while matches!(protocol, 0 | 43 | 44 | 51 | 60) {
        if protocol == 44 {
            return Err(PacketError::Fragmented);
        }
        if extension_count == MAX_EXTENSION_HEADERS {
            return Err(PacketError::Unsupported);
        }
        if remaining.len() < 2 {
            return Err(PacketError::Malformed);
        }
        let length = if protocol == 51 {
            (usize::from(remaining[1]) + 2) * 4
        } else {
            (usize::from(remaining[1]) + 1) * 8
        };
        if length > remaining.len() {
            return Err(PacketError::Malformed);
        }
        protocol = remaining[0];
        remaining = &remaining[length..];
        extension_count += 1;
    }
    packet.ip_protocol = Some(protocol);
    transport::parse(remaining, packet)
}
