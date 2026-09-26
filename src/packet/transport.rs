use super::{PacketError, PacketMetadata, TCP_PROTOCOL, UDP_PROTOCOL};

const TCP_MIN_HEADER_LENGTH: usize = 20;
const UDP_HEADER_LENGTH: usize = 8;

pub(super) fn parse(payload: &[u8], packet: &mut PacketMetadata) -> Result<(), PacketError> {
    match packet.ip_protocol {
        Some(TCP_PROTOCOL) => {
            if payload.len() < TCP_MIN_HEADER_LENGTH {
                return Err(PacketError::Malformed);
            }
            let header_length = usize::from(payload[12] >> 4) * 4;
            if header_length < TCP_MIN_HEADER_LENGTH || header_length > payload.len() {
                return Err(PacketError::Malformed);
            }
        },
        Some(UDP_PROTOCOL) => {
            if payload.len() < UDP_HEADER_LENGTH {
                return Err(PacketError::Malformed);
            }
            let length = usize::from(u16::from_be_bytes([payload[4], payload[5]]));
            if length < UDP_HEADER_LENGTH || length != payload.len() {
                return Err(PacketError::Malformed);
            }
        },
        _ => return Ok(()),
    }
    packet.src_port = Some(u16::from_be_bytes([payload[0], payload[1]]));
    packet.dst_port = Some(u16::from_be_bytes([payload[2], payload[3]]));
    Ok(())
}
