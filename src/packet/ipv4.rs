use super::{transport, PacketError, PacketMetadata};
use std::net::{IpAddr, Ipv4Addr};

// IPv4の固定部分は20バイト。IHLの値だけを信用して添字に使わない。
const MIN_HEADER_LENGTH: usize = 20;
const FRAGMENT_MASK: u16 = 0x3fff;

pub(super) fn parse(payload: &[u8], packet: &mut PacketMetadata) -> Result<(), PacketError> {
    if payload.len() < MIN_HEADER_LENGTH || payload[0] >> 4 != 4 {
        return Err(PacketError::Malformed);
    }
    let header_length = usize::from(payload[0] & 0x0f) * 4;
    let total_length = usize::from(u16::from_be_bytes([payload[2], payload[3]]));
    if header_length < MIN_HEADER_LENGTH || total_length < header_length || total_length > payload.len() {
        return Err(PacketError::Malformed);
    }
    if u16::from_be_bytes([payload[6], payload[7]]) & FRAGMENT_MASK != 0 {
        return Err(PacketError::Fragmented);
    }
    packet.src_ip = Some(IpAddr::V4(Ipv4Addr::new(payload[12], payload[13], payload[14], payload[15])));
    packet.dst_ip = Some(IpAddr::V4(Ipv4Addr::new(payload[16], payload[17], payload[18], payload[19])));
    packet.ip_protocol = Some(payload[9]);
    // EthernetのパディングをTCP/UDPデータとして解釈しない。
    transport::parse(&payload[header_length..total_length], packet)
}
