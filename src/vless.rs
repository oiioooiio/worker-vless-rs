use bytes::{Buf, Bytes};
use std::net::{Ipv4Addr, Ipv6Addr};
use uuid::Uuid;

#[derive(Debug)]
pub enum Command {
    Tcp = 0x01,
    Udp = 0x02,
    Mux = 0x03,
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Tcp => write!(f, "tcp"),
            Command::Udp => write!(f, "udp"),
            Command::Mux => write!(f, "mux"),
        }
    }
}

impl From<u8> for Command {
    fn from(cmd: u8) -> Self {
        match cmd {
            0x01 => Self::Tcp,
            0x02 => Self::Udp,
            0x03 => Self::Mux,
            _ => panic!("invalid command"),
        }
    }
}

#[derive(Debug)]
pub enum Address {
    IPv4(Ipv4Addr),
    Domain(String),
    IPv6(Ipv6Addr),
}

impl ToString for Address {
    fn to_string(&self) -> String {
        match self {
            Address::IPv4(addr) => addr.to_string(),
            Address::Domain(domain) => domain.clone(),
            Address::IPv6(addr) => addr.to_string(),
        }
    }
}

pub struct Request {
    pub version: u8,
    pub uuid: Uuid,
    pub extra: Bytes,
    pub cmd: Command,
    pub port: u16,
    pub addr: Address,
    pub payload: Bytes,
}

impl Request {
    pub fn parse_from(mut payload: Bytes, env_uuid: Uuid) -> Result<Self, &'static str> {
        if payload.len() < 18 {
            return Err("invalid request length");
        }

        let version = payload.get_u8();

        if version != 0x00 {
            return Err("invalid version");
        }

        let uuid = Uuid::from_u128(payload.get_u128());

        if uuid != env_uuid {
            return Err("invalid uuid");
        }

        // trust the client since uuid is verified
        // crash if the request is malformed
        let extra_len = payload.get_u8() as usize;
        let extra = payload.split_to(extra_len);
        let cmd = payload.get_u8().into();
        let port = payload.get_u16();
        let addr_type = payload.get_u8();
        let addr = match addr_type {
            1 => Address::IPv4(Ipv4Addr::from(payload.get_u32())),
            2 => {
                let len = payload.get_u8() as usize;
                let domain = String::from_utf8(payload.split_to(len).to_vec()).unwrap();
                Address::Domain(domain)
            }
            3 => Address::IPv6(Ipv6Addr::from(payload.get_u128())),
            _ => panic!("invalid address type"),
        };
        Ok(Self {
            version,
            uuid,
            extra,
            cmd,
            port,
            addr,
            payload,
        })
    }
}

#[cfg(debug_assertions)]
impl std::fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Message")
            .field("version", &self.version)
            .field("uuid", &self.uuid)
            .field("cmd", &self.cmd)
            .field("port", &self.port)
            .field("addr", &self.addr)
            .finish()
    }
}
