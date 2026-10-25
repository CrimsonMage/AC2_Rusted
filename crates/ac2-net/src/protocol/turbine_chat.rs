//! The Turbine chat client's protocol (chatclient.dll, Legions build), carried in queue-4
//! `0x100D3` events (`CChatEvent::CreateForSend` out, `Turbine::Client::ProcessLogonEventQueue`
//! in): room chat between the player and the chat server.
//!
//! A packet is `ChatNetworkBlobHeader::Serialize` (seven u32s) then the payload
//! (`Serializer::SerializeObject(SmartBuffer)`: a u32 size, then the bytes). The chat archive
//! is not word aligned (`ArchiveParameters` starts as 1, without the aligned bit), so nothing
//! pads. Strings are `PStringBase<unsigned short>::Serialize`: a compressed length
//! (`SB_As32Bit_Compressed`), then the UTF-16 units, plain (no munging, no terminator).

use crate::protocol::reader::{Reader, Writer};
use crate::protocol::ProtocolError;

/// `ChatNetworkBlobType`.
pub mod blob_type {
    pub const EVENT_BINARY: u32 = 1;
    pub const REQUEST_BINARY: u32 = 3;
    pub const RESPONSE_BINARY: u32 = 5;
}

/// Method ids (`SendToRoomByNameMethod` 1, `SendToRoomByIDMethod` 2): the request's dispatch
/// type and its response's.
pub const METHOD_SEND_TO_ROOM_BY_NAME: u32 = 1;
pub const METHOD_SEND_TO_ROOM_BY_ID: u32 = 2;
/// Event dispatch type of `SendToRoomChatEvent` (`IChatClientImpl::DispatchEvent`).
pub const EVENT_SEND_TO_ROOM: u32 = 1;

/// `ChatNetworkBlobHeader`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChatHeader {
    pub blob_type: u32,
    pub dispatch_type: u32,
    pub target_type: u32,
    pub target_id: u32,
    pub transport_type: u32,
    pub transport_id: u32,
    pub cookie: u32,
}

/// One packet: the header and its payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatPacket {
    pub header: ChatHeader,
    pub payload: Vec<u8>,
}

/// A request the client sends (`IChatClientImpl::Invoke`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatRequest {
    /// `SendToRoomByNameRequest::Serialize`.
    SendToRoomByName {
        context: u32,
        room: String,
        text: String,
        extra: Vec<u8>,
    },
    /// `SendToRoomByIDRequest::Serialize`.
    SendToRoomById {
        context: u32,
        room: u32,
        text: String,
        extra: Vec<u8>,
    },
}

/// `SendToRoomByNameResponse::Serialize` (the by-id response is the same layout).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChatResponse {
    pub context: u32,
    pub response_id: u32,
    pub method: u32,
    pub result: u32,
}

/// `SendToRoomChatEvent::Serialize`: what someone said in a room.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomChat {
    pub room: u32,
    pub speaker: String,
    pub text: String,
    /// The sender's `remoteBlob` (`GunChatExtraInfo` from AC2 clients).
    pub extra: Vec<u8>,
}

/// A decoded packet from the chat server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChatMessage {
    /// `IChatClientImpl::Dispatch` (response blob, by-name or by-id).
    Response(ChatResponse),
    /// `IChatClientImpl::DispatchEvent` (event blob, dispatch 1).
    RoomChat(RoomChat),
}

/// `GunChatExtraInfo` (AC2's `remoteBlob` on room chat, `GunChatExtraInfo::PackIntoBlob`):
/// AC2 `StreamPack`, so 4-aligned and with the AC2 string munging, unlike the chat archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GunChatExtraInfo {
    pub version: u32,
    pub misc_flags: u32,
    /// The script's weenie chat flags (`RequestGUN_SendToRoom`'s third argument).
    pub weenie_flags: u32,
    /// `StringTableMetaLanguage::RetrieveMetaLetters` of the sender's name.
    pub meta_letters: String,
}

impl GunChatExtraInfo {
    /// `GunChatExtraInfo::GunChatExtraInfo`: version 1, no misc flags.
    pub fn new(weenie_flags: u32, meta_letters: String) -> Self {
        Self {
            version: 1,
            misc_flags: 0,
            weenie_flags,
            meta_letters,
        }
    }

    /// `GunChatExtraInfo::StreamPack` (pack).
    pub fn encode(&self) -> Vec<u8> {
        let mut w = Writer::new();
        w.u32(self.version)
            .u32(self.misc_flags)
            .u32(self.weenie_flags);
        w.wide_string(&self.meta_letters);
        w.bytes
    }

    /// `GunChatExtraInfo::StreamPack` (unpack): the flags and letters only for version 1.
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() < 8 {
            return Err(ProtocolError::Truncated);
        }
        let mut r = Reader::new(bytes);
        let mut info = Self {
            version: r.u32()?,
            misc_flags: r.u32()?,
            weenie_flags: 0,
            meta_letters: String::new(),
        };
        if info.version == 1 {
            info.weenie_flags = r.u32()?;
            info.meta_letters = r.wide_string()?;
        }
        Ok(info)
    }
}

/// The chat archive: no alignment, little-endian.
#[derive(Default)]
struct Out(Vec<u8>);

impl Out {
    fn u32(&mut self, v: u32) {
        self.0.extend(v.to_le_bytes());
    }
    /// `SB_As32Bit_Compressed::Serialize`: one byte below 0x80, two (high byte | 0x80) below
    /// 0x4000, else the top byte | 0xC0, the next byte, then the low 16 bits.
    fn compressed(&mut self, v: u32) {
        if v < 0x80 {
            self.0.push(v as u8);
        } else if v < 0x4000 {
            self.0.extend([(v >> 8) as u8 | 0x80, v as u8]);
        } else {
            self.0.extend([(v >> 24) as u8 | 0xC0, (v >> 16) as u8]);
            self.0.extend((v as u16).to_le_bytes());
        }
    }
    fn string(&mut self, s: &str) {
        let units: Vec<u16> = s.encode_utf16().collect();
        self.compressed(units.len() as u32);
        for unit in units {
            self.0.extend(unit.to_le_bytes());
        }
    }
    fn blob(&mut self, b: &[u8]) {
        self.u32(b.len() as u32);
        self.0.extend_from_slice(b);
    }
}

struct In<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> In<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ProtocolError> {
        let end = self.pos.checked_add(n).ok_or(ProtocolError::Truncated)?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or(ProtocolError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }
    fn u8(&mut self) -> Result<u8, ProtocolError> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, ProtocolError> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn compressed(&mut self) -> Result<u32, ProtocolError> {
        let b0 = u32::from(self.u8()?);
        if b0 & 0x80 == 0 {
            return Ok(b0);
        }
        let b1 = u32::from(self.u8()?);
        if b0 & 0x40 == 0 {
            return Ok((b0 & 0x7F) << 8 | b1);
        }
        let low = u32::from(u16::from_le_bytes(self.take(2)?.try_into().unwrap()));
        Ok(((b0 & 0x3F) << 8 | b1) << 16 | low)
    }
    fn string(&mut self) -> Result<String, ProtocolError> {
        let units = self.compressed()? as usize;
        let data = self.take(units.checked_mul(2).ok_or(ProtocolError::Truncated)?)?;
        let units: Vec<u16> = data
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&units).map_err(|_| ProtocolError::Invalid("chat string"))
    }
    fn blob(&mut self) -> Result<Vec<u8>, ProtocolError> {
        let size = self.u32()? as usize;
        Ok(self.take(size)?.to_vec())
    }
    fn finish(&self) -> Result<(), ProtocolError> {
        if self.pos == self.bytes.len() {
            Ok(())
        } else {
            Err(ProtocolError::TrailingBytes)
        }
    }
}

impl ChatPacket {
    /// `IChatClientImpl::SendNetPacket` / `ChatNetworkBlob::UnpackFromNetPacket`.
    pub fn encode(&self) -> Vec<u8> {
        let h = self.header;
        let mut out = Out::default();
        for v in [
            h.blob_type,
            h.dispatch_type,
            h.target_type,
            h.target_id,
            h.transport_type,
            h.transport_id,
            h.cookie,
        ] {
            out.u32(v);
        }
        out.blob(&self.payload);
        out.0
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let mut r = In { bytes, pos: 0 };
        let header = ChatHeader {
            blob_type: r.u32()?,
            dispatch_type: r.u32()?,
            target_type: r.u32()?,
            target_id: r.u32()?,
            transport_type: r.u32()?,
            transport_id: r.u32()?,
            cookie: r.u32()?,
        };
        let payload = r.blob()?;
        r.finish()?;
        Ok(Self { header, payload })
    }
}

impl ChatRequest {
    fn method(&self) -> u32 {
        match self {
            Self::SendToRoomByName { .. } => METHOD_SEND_TO_ROOM_BY_NAME,
            Self::SendToRoomById { .. } => METHOD_SEND_TO_ROOM_BY_ID,
        }
    }

    /// The packet `IChatClientImpl::Invoke` sends: a binary request (3) dispatched by method,
    /// target type 1, the rest 0.
    pub fn to_packet(&self) -> ChatPacket {
        let mut out = Out::default();
        // `Request`: the context id and the method (twice: the base's word and `m_methodID`).
        let (context, method) = match self {
            Self::SendToRoomByName { context, .. } | Self::SendToRoomById { context, .. } => {
                (*context, self.method())
            }
        };
        out.u32(context);
        out.u32(method);
        out.u32(method);
        match self {
            Self::SendToRoomByName {
                room, text, extra, ..
            } => {
                out.string(room);
                out.string(text);
                out.blob(extra);
            }
            Self::SendToRoomById {
                room, text, extra, ..
            } => {
                out.u32(*room);
                out.string(text);
                out.blob(extra);
            }
        }
        ChatPacket {
            header: ChatHeader {
                blob_type: blob_type::REQUEST_BINARY,
                dispatch_type: method,
                target_type: 1,
                ..ChatHeader::default()
            },
            payload: out.0,
        }
    }

    /// The server's side.
    pub fn from_packet(packet: &ChatPacket) -> Result<Self, ProtocolError> {
        if packet.header.blob_type != blob_type::REQUEST_BINARY {
            return Err(ProtocolError::Invalid("chat request blob type"));
        }
        let mut r = In {
            bytes: &packet.payload,
            pos: 0,
        };
        let context = r.u32()?;
        let method = r.u32()?;
        if r.u32()? != method || method != packet.header.dispatch_type {
            return Err(ProtocolError::Invalid("chat request method"));
        }
        let request = match method {
            METHOD_SEND_TO_ROOM_BY_NAME => Self::SendToRoomByName {
                context,
                room: r.string()?,
                text: r.string()?,
                extra: r.blob()?,
            },
            METHOD_SEND_TO_ROOM_BY_ID => Self::SendToRoomById {
                context,
                room: r.u32()?,
                text: r.string()?,
                extra: r.blob()?,
            },
            _ => return Err(ProtocolError::Unsupported("chat request method")),
        };
        r.finish()?;
        Ok(request)
    }
}

impl ChatMessage {
    /// `IChatClientImpl::OnNetworkData`: responses and room events (a request from the server
    /// has no handler in the client and is refused).
    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        let packet = ChatPacket::decode(bytes)?;
        let mut r = In {
            bytes: &packet.payload,
            pos: 0,
        };
        let message = match (packet.header.blob_type, packet.header.dispatch_type) {
            (
                blob_type::RESPONSE_BINARY,
                METHOD_SEND_TO_ROOM_BY_NAME | METHOD_SEND_TO_ROOM_BY_ID,
            ) => Self::Response(ChatResponse {
                context: r.u32()?,
                response_id: r.u32()?,
                method: r.u32()?,
                result: r.u32()?,
            }),
            (blob_type::EVENT_BINARY, EVENT_SEND_TO_ROOM) => Self::RoomChat(RoomChat {
                room: r.u32()?,
                speaker: r.string()?,
                text: r.string()?,
                extra: r.blob()?,
            }),
            _ => return Err(ProtocolError::Unsupported("chat blob")),
        };
        r.finish()?;
        Ok(message)
    }

    /// The server's side: the packet for this message.
    pub fn to_packet(&self) -> ChatPacket {
        let mut out = Out::default();
        let (blob_type, dispatch_type) = match self {
            Self::Response(response) => {
                out.u32(response.context);
                out.u32(response.response_id);
                out.u32(response.method);
                out.u32(response.result);
                (blob_type::RESPONSE_BINARY, response.method)
            }
            Self::RoomChat(chat) => {
                out.u32(chat.room);
                out.string(&chat.speaker);
                out.string(&chat.text);
                out.blob(&chat.extra);
                (blob_type::EVENT_BINARY, EVENT_SEND_TO_ROOM)
            }
        };
        ChatPacket {
            header: ChatHeader {
                blob_type,
                dispatch_type,
                ..ChatHeader::default()
            },
            payload: out.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gun_chat_extra_info_is_an_ac2_stream_pack() {
        let info = GunChatExtraInfo::new(0x4, String::new());
        let bytes = info.encode();
        assert_eq!(
            bytes,
            [1u32, 0, 4, 0]
                .iter()
                .flat_map(|v| v.to_le_bytes())
                .collect::<Vec<_>>()
        );
        assert_eq!(GunChatExtraInfo::decode(&bytes), Ok(info));
        let named = GunChatExtraInfo::new(0, "m".into());
        assert_eq!(GunChatExtraInfo::decode(&named.encode()), Ok(named));
    }

    #[test]
    fn lengths_compress_like_sb_as32bit() {
        for (v, bytes) in [
            (5u32, vec![5u8]),
            (0x7F, vec![0x7F]),
            (0x80, vec![0x80, 0x80]),
            (0x3FFF, vec![0xBF, 0xFF]),
            (0x12_3456, vec![0xC0, 0x12, 0x56, 0x34]),
        ] {
            let mut out = Out::default();
            out.compressed(v);
            assert_eq!(out.0, bytes, "{v:#x}");
            let mut r = In {
                bytes: &bytes,
                pos: 0,
            };
            assert_eq!(r.compressed().unwrap(), v);
        }
    }

    #[test]
    fn a_room_message_is_a_binary_request_by_method() {
        let request = ChatRequest::SendToRoomById {
            context: 1,
            room: 0x10,
            text: "Hi".into(),
            extra: vec![9, 8],
        };
        let bytes = request.to_packet().encode();
        let mut expected: Vec<u8> = [3u32, 2, 1, 0, 0, 0, 0, 27, 1, 2, 2, 0x10]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        expected.extend([2, b'H', 0, b'i', 0]);
        expected.extend(2u32.to_le_bytes());
        expected.extend([9, 8]);
        assert_eq!(bytes, expected);
        assert_eq!(
            ChatRequest::from_packet(&ChatPacket::decode(&bytes).unwrap()),
            Ok(request)
        );
        let by_name = ChatRequest::SendToRoomByName {
            context: 2,
            room: "General".into(),
            text: "x".into(),
            extra: vec![],
        };
        let packet = by_name.to_packet();
        assert_eq!(packet.header.dispatch_type, 1);
        assert_eq!(ChatRequest::from_packet(&packet), Ok(by_name));
    }

    #[test]
    fn events_and_responses_decode_and_truncation_fails() {
        let chat = ChatMessage::RoomChat(RoomChat {
            room: 7,
            speaker: "Birch".into(),
            text: "hello".into(),
            extra: vec![1, 2, 3],
        });
        let bytes = chat.to_packet().encode();
        assert_eq!(ChatMessage::decode(&bytes), Ok(chat));
        for end in 0..bytes.len() {
            assert!(ChatMessage::decode(&bytes[..end]).is_err(), "{end}");
        }
        let response = ChatMessage::Response(ChatResponse {
            context: 1,
            response_id: 4,
            method: 2,
            result: 0,
        });
        assert_eq!(
            ChatMessage::decode(&response.to_packet().encode()),
            Ok(response)
        );
    }
}
