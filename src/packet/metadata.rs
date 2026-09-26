use serde::{Deserialize, Deserializer};
use std::{fmt, net::IpAddr, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MacAddress(pub [u8; 6]);

impl fmt::Display for MacAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let [a, b, c, d, e, f] = self.0;
        write!(formatter, "{a:02x}:{b:02x}:{c:02x}:{d:02x}:{e:02x}:{f:02x}")
    }
}

impl FromStr for MacAddress {
    type Err = &'static str;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut bytes = [0; 6];
        let parts: Vec<_> = value.split([':', '-']).collect();
        if parts.len() != bytes.len() {
            return Err("MACアドレスは6オクテットで指定してください");
        }
        for (byte, part) in bytes.iter_mut().zip(parts) {
            if part.len() != 2 {
                return Err("MACアドレスの各オクテットは16進数2桁で指定してください");
            }
            *byte = u8::from_str_radix(part, 16).map_err(|_| "MACアドレスの書式が不正です")?;
        }
        Ok(Self(bytes))
    }
}

impl<'de> Deserialize<'de> for MacAddress {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug)]
pub struct PacketMetadata {
    pub src_mac: MacAddress,
    pub dst_mac: MacAddress,
    pub ether_type: u16,
    pub src_ip: Option<IpAddr>,
    pub dst_ip: Option<IpAddr>,
    pub ip_protocol: Option<u8>,
    pub src_port: Option<u16>,
    pub dst_port: Option<u16>,
}
