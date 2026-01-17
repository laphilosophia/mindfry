//! MFBP Codec - Binary encoding/decoding
//!
//! Frame format: [u32 length][u8 opcode][payload...]
//!
//! All integers are little-endian.

use std::io;

use super::{Event, LineageInfo, OpCode, Request, Response, ResponseData};

/// MFBP protocol errors
#[derive(Debug)]
pub enum MfbpError {
    /// IO error
    Io(io::Error),
    /// Invalid OpCode
    InvalidOpCode(u8),
    /// Payload too short
    PayloadTooShort,
    /// Invalid UTF-8 in string
    InvalidUtf8,
    /// Frame too large
    FrameTooLarge,
}

impl From<io::Error> for MfbpError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::fmt::Display for MfbpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {}", e),
            Self::InvalidOpCode(op) => write!(f, "Invalid OpCode: 0x{:02X}", op),
            Self::PayloadTooShort => write!(f, "Payload too short"),
            Self::InvalidUtf8 => write!(f, "Invalid UTF-8 in payload"),
            Self::FrameTooLarge => write!(f, "Frame exceeds maximum size"),
        }
    }
}

impl std::error::Error for MfbpError {}

/// Maximum frame size (16 MB)
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

/// MFBP Codec for encoding/decoding messages
pub struct MfbpCodec;

impl MfbpCodec {
    // ═══════════════════════════════════════════════════════════════
    // ENCODING
    // ═══════════════════════════════════════════════════════════════

    /// Encode a request to bytes
    pub fn encode_request(request: &Request) -> Vec<u8> {
        let mut payload = Vec::new();
        let opcode = request.opcode();

        match request {
            Request::LineageCreate {
                id,
                energy,
                threshold,
                decay_rate,
            } => {
                Self::write_string(&mut payload, id);
                payload.extend_from_slice(&energy.to_le_bytes());
                payload.extend_from_slice(&threshold.to_le_bytes());
                payload.extend_from_slice(&decay_rate.to_le_bytes());
            }
            Request::LineageGet { id }
            | Request::LineageForget { id }
            | Request::LineageTouch { id }
            | Request::BondNeighbors { id } => {
                Self::write_string(&mut payload, id);
            }
            Request::LineageStimulate { id, delta } => {
                Self::write_string(&mut payload, id);
                payload.extend_from_slice(&delta.to_le_bytes());
            }
            Request::BondConnect {
                source,
                target,
                strength,
            } => {
                Self::write_string(&mut payload, source);
                Self::write_string(&mut payload, target);
                payload.extend_from_slice(&strength.to_le_bytes());
            }
            Request::BondReinforce {
                source,
                target,
                delta,
            } => {
                Self::write_string(&mut payload, source);
                Self::write_string(&mut payload, target);
                payload.extend_from_slice(&delta.to_le_bytes());
            }
            Request::BondSever { source, target } => {
                Self::write_string(&mut payload, source);
                Self::write_string(&mut payload, target);
            }
            Request::QueryConscious { min_energy } => {
                payload.extend_from_slice(&min_energy.to_le_bytes());
            }
            Request::QueryTopK { k } => {
                payload.extend_from_slice(&k.to_le_bytes());
            }
            Request::QueryTrauma { min_rigidity } => {
                payload.extend_from_slice(&min_rigidity.to_le_bytes());
            }
            Request::QueryPattern { pattern } => {
                Self::write_string(&mut payload, pattern);
            }
            Request::Ping | Request::Stats | Request::Unsubscribe => {
                // No payload
            }
            Request::Snapshot { name } | Request::Restore { name } => {
                Self::write_string(&mut payload, name);
            }
            Request::Freeze { frozen } => {
                payload.push(if *frozen { 1 } else { 0 });
            }
            Request::PhysicsTune { param, value } => {
                payload.push(*param);
                payload.extend_from_slice(&value.to_le_bytes());
            }
            Request::MoodSet { mood } => {
                payload.extend_from_slice(&mood.to_le_bytes());
            }
            Request::Subscribe { events_mask } => {
                payload.extend_from_slice(&events_mask.to_le_bytes());
            }
        }

        Self::wrap_frame(opcode, &payload)
    }

    /// Encode a response to bytes
    pub fn encode_response(response: &Response) -> Vec<u8> {
        let mut payload = Vec::new();
        let opcode = match response {
            Response::Ok(_) => OpCode::ResponseOk,
            Response::Error { .. } => OpCode::ResponseError,
            Response::Event(_) => OpCode::ResponseEvent,
        };

        match response {
            Response::Ok(data) => {
                Self::encode_response_data(&mut payload, data);
            }
            Response::Error { code, message } => {
                payload.push(*code as u8);
                Self::write_string(&mut payload, message);
            }
            Response::Event(event) => {
                Self::encode_event(&mut payload, event);
            }
        }

        Self::wrap_frame(opcode, &payload)
    }

    fn encode_response_data(buf: &mut Vec<u8>, data: &ResponseData) {
        match data {
            ResponseData::Ack => {
                buf.push(0x00);
            }
            ResponseData::Pong => {
                buf.push(0x01);
            }
            ResponseData::Lineage(info) => {
                buf.push(0x02);
                Self::encode_lineage_info(buf, info);
            }
            ResponseData::Lineages(list) => {
                buf.push(0x03);
                buf.extend_from_slice(&(list.len() as u32).to_le_bytes());
                for info in list {
                    Self::encode_lineage_info(buf, info);
                }
            }
            ResponseData::Neighbors(list) => {
                buf.push(0x04);
                buf.extend_from_slice(&(list.len() as u32).to_le_bytes());
                for neighbor in list {
                    Self::write_string(buf, &neighbor.id);
                    buf.extend_from_slice(&neighbor.bond_strength.to_le_bytes());
                    buf.push(if neighbor.is_learned { 1 } else { 0 });
                }
            }
            ResponseData::Stats(stats) => {
                buf.push(0x05);
                buf.extend_from_slice(&(stats.lineage_count as u32).to_le_bytes());
                buf.extend_from_slice(&(stats.bond_count as u32).to_le_bytes());
                buf.extend_from_slice(&(stats.conscious_count as u32).to_le_bytes());
                buf.extend_from_slice(&stats.total_energy.to_le_bytes());
                buf.push(if stats.is_frozen { 1 } else { 0 });
                buf.extend_from_slice(&stats.uptime_secs.to_le_bytes());
            }
            ResponseData::SnapshotCreated { name } => {
                buf.push(0x06);
                Self::write_string(buf, name);
            }
        }
    }

    fn encode_lineage_info(buf: &mut Vec<u8>, info: &LineageInfo) {
        Self::write_string(buf, &info.id);
        buf.extend_from_slice(&info.energy.to_le_bytes());
        buf.extend_from_slice(&info.threshold.to_le_bytes());
        buf.extend_from_slice(&info.decay_rate.to_le_bytes());
        buf.extend_from_slice(&info.rigidity.to_le_bytes());
        buf.push(if info.is_conscious { 1 } else { 0 });
        buf.extend_from_slice(&info.last_access_ms.to_le_bytes());
    }

    fn encode_event(buf: &mut Vec<u8>, event: &Event) {
        match event {
            Event::LineageCreated { id, energy } => {
                buf.push(0x01);
                Self::write_string(buf, id);
                buf.extend_from_slice(&energy.to_le_bytes());
            }
            Event::LineageStimulated {
                id,
                new_energy,
                delta,
            } => {
                buf.push(0x02);
                Self::write_string(buf, id);
                buf.extend_from_slice(&new_energy.to_le_bytes());
                buf.extend_from_slice(&delta.to_le_bytes());
            }
            Event::LineageForgotten { id } => {
                buf.push(0x03);
                Self::write_string(buf, id);
            }
            Event::BondCreated {
                source,
                target,
                strength,
            } => {
                buf.push(0x04);
                Self::write_string(buf, source);
                Self::write_string(buf, target);
                buf.extend_from_slice(&strength.to_le_bytes());
            }
            Event::BondSevered { source, target } => {
                buf.push(0x05);
                Self::write_string(buf, source);
                Self::write_string(buf, target);
            }
            Event::DecayTick {
                processed,
                dead_count,
            } => {
                buf.push(0x06);
                buf.extend_from_slice(&(*processed as u32).to_le_bytes());
                buf.extend_from_slice(&(*dead_count as u32).to_le_bytes());
            }
            Event::SnapshotCreated { name } => {
                buf.push(0x07);
                Self::write_string(buf, name);
            }
        }
    }

    fn wrap_frame(opcode: OpCode, payload: &[u8]) -> Vec<u8> {
        let total_len = 1 + payload.len();
        let mut frame = Vec::with_capacity(4 + total_len);
        frame.extend_from_slice(&(total_len as u32).to_le_bytes());
        frame.push(opcode.as_byte());
        frame.extend_from_slice(payload);
        frame
    }

    fn write_string(buf: &mut Vec<u8>, s: &str) {
        let bytes = s.as_bytes();
        buf.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        buf.extend_from_slice(bytes);
    }

    // ═══════════════════════════════════════════════════════════════
    // DECODING
    // ═══════════════════════════════════════════════════════════════

    /// Decode a request from bytes
    pub fn decode_request(frame: &[u8]) -> Result<Request, MfbpError> {
        if frame.len() < 5 {
            return Err(MfbpError::PayloadTooShort);
        }

        let len = u32::from_le_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize;
        if len > MAX_FRAME_SIZE {
            return Err(MfbpError::FrameTooLarge);
        }

        let opcode_byte = frame[4];
        let opcode = OpCode::from_byte(opcode_byte).ok_or(MfbpError::InvalidOpCode(opcode_byte))?;

        let payload = &frame[5..];
        let mut cursor = 0;

        let request = match opcode {
            OpCode::LineageCreate => {
                let id = Self::read_string(payload, &mut cursor)?;
                let energy = Self::read_f32(payload, &mut cursor)?;
                let threshold = Self::read_f32(payload, &mut cursor)?;
                let decay_rate = Self::read_f32(payload, &mut cursor)?;
                Request::LineageCreate {
                    id,
                    energy,
                    threshold,
                    decay_rate,
                }
            }
            OpCode::LineageGet => {
                let id = Self::read_string(payload, &mut cursor)?;
                Request::LineageGet { id }
            }
            OpCode::LineageStimulate => {
                let id = Self::read_string(payload, &mut cursor)?;
                let delta = Self::read_f32(payload, &mut cursor)?;
                Request::LineageStimulate { id, delta }
            }
            OpCode::LineageForget => {
                let id = Self::read_string(payload, &mut cursor)?;
                Request::LineageForget { id }
            }
            OpCode::LineageTouch => {
                let id = Self::read_string(payload, &mut cursor)?;
                Request::LineageTouch { id }
            }
            OpCode::BondConnect => {
                let source = Self::read_string(payload, &mut cursor)?;
                let target = Self::read_string(payload, &mut cursor)?;
                let strength = Self::read_f32(payload, &mut cursor)?;
                Request::BondConnect {
                    source,
                    target,
                    strength,
                }
            }
            OpCode::BondReinforce => {
                let source = Self::read_string(payload, &mut cursor)?;
                let target = Self::read_string(payload, &mut cursor)?;
                let delta = Self::read_f32(payload, &mut cursor)?;
                Request::BondReinforce {
                    source,
                    target,
                    delta,
                }
            }
            OpCode::BondSever => {
                let source = Self::read_string(payload, &mut cursor)?;
                let target = Self::read_string(payload, &mut cursor)?;
                Request::BondSever { source, target }
            }
            OpCode::BondNeighbors => {
                let id = Self::read_string(payload, &mut cursor)?;
                Request::BondNeighbors { id }
            }
            OpCode::QueryConscious => {
                let min_energy = Self::read_f32(payload, &mut cursor)?;
                Request::QueryConscious { min_energy }
            }
            OpCode::QueryTopK => {
                let k = Self::read_u32(payload, &mut cursor)?;
                Request::QueryTopK { k }
            }
            OpCode::QueryTrauma => {
                let min_rigidity = Self::read_f32(payload, &mut cursor)?;
                Request::QueryTrauma { min_rigidity }
            }
            OpCode::QueryPattern => {
                let pattern = Self::read_string(payload, &mut cursor)?;
                Request::QueryPattern { pattern }
            }
            OpCode::SysPing => Request::Ping,
            OpCode::SysStats => Request::Stats,
            OpCode::SysSnapshot => {
                let name = Self::read_string(payload, &mut cursor)?;
                Request::Snapshot { name }
            }
            OpCode::SysRestore => {
                let name = Self::read_string(payload, &mut cursor)?;
                Request::Restore { name }
            }
            OpCode::SysFreeze => {
                let frozen = Self::read_u8(payload, &mut cursor)? != 0;
                Request::Freeze { frozen }
            }
            OpCode::PhysicsTune => {
                let param = Self::read_u8(payload, &mut cursor)?;
                let value = Self::read_f32(payload, &mut cursor)?;
                Request::PhysicsTune { param, value }
            }
            OpCode::SysMoodSet => {
                let mood = Self::read_f32(payload, &mut cursor)?;
                Request::MoodSet { mood }
            }
            OpCode::StreamSubscribe => {
                let events_mask = Self::read_u32(payload, &mut cursor)?;
                Request::Subscribe { events_mask }
            }
            OpCode::StreamUnsubscribe => Request::Unsubscribe,
            _ => return Err(MfbpError::InvalidOpCode(opcode_byte)),
        };

        Ok(request)
    }

    // Helper read functions
    fn read_string(buf: &[u8], cursor: &mut usize) -> Result<String, MfbpError> {
        if *cursor + 2 > buf.len() {
            return Err(MfbpError::PayloadTooShort);
        }
        let len = u16::from_le_bytes([buf[*cursor], buf[*cursor + 1]]) as usize;
        *cursor += 2;

        if *cursor + len > buf.len() {
            return Err(MfbpError::PayloadTooShort);
        }
        let s = std::str::from_utf8(&buf[*cursor..*cursor + len])
            .map_err(|_| MfbpError::InvalidUtf8)?
            .to_string();
        *cursor += len;
        Ok(s)
    }

    fn read_f32(buf: &[u8], cursor: &mut usize) -> Result<f32, MfbpError> {
        if *cursor + 4 > buf.len() {
            return Err(MfbpError::PayloadTooShort);
        }
        let v = f32::from_le_bytes([
            buf[*cursor],
            buf[*cursor + 1],
            buf[*cursor + 2],
            buf[*cursor + 3],
        ]);
        *cursor += 4;
        Ok(v)
    }

    fn read_u32(buf: &[u8], cursor: &mut usize) -> Result<u32, MfbpError> {
        if *cursor + 4 > buf.len() {
            return Err(MfbpError::PayloadTooShort);
        }
        let v = u32::from_le_bytes([
            buf[*cursor],
            buf[*cursor + 1],
            buf[*cursor + 2],
            buf[*cursor + 3],
        ]);
        *cursor += 4;
        Ok(v)
    }

    fn read_u8(buf: &[u8], cursor: &mut usize) -> Result<u8, MfbpError> {
        if *cursor >= buf.len() {
            return Err(MfbpError::PayloadTooShort);
        }
        let v = buf[*cursor];
        *cursor += 1;
        Ok(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_ping() {
        let request = Request::Ping;
        let encoded = MfbpCodec::encode_request(&request);
        let decoded = MfbpCodec::decode_request(&encoded).unwrap();

        match decoded {
            Request::Ping => {}
            _ => panic!("Expected Ping"),
        }
    }

    #[test]
    fn test_encode_decode_lineage_create() {
        let request = Request::LineageCreate {
            id: "test_concept".into(),
            energy: 0.85,
            threshold: 0.5,
            decay_rate: 0.001,
        };
        let encoded = MfbpCodec::encode_request(&request);
        let decoded = MfbpCodec::decode_request(&encoded).unwrap();

        match decoded {
            Request::LineageCreate {
                id,
                energy,
                threshold,
                decay_rate,
            } => {
                assert_eq!(id, "test_concept");
                assert!((energy - 0.85).abs() < 0.001);
                assert!((threshold - 0.5).abs() < 0.001);
                assert!((decay_rate - 0.001).abs() < 0.0001);
            }
            _ => panic!("Expected LineageCreate"),
        }
    }

    #[test]
    fn test_encode_decode_bond_connect() {
        let request = Request::BondConnect {
            source: "fire".into(),
            target: "heat".into(),
            strength: 0.9,
        };
        let encoded = MfbpCodec::encode_request(&request);
        let decoded = MfbpCodec::decode_request(&encoded).unwrap();

        match decoded {
            Request::BondConnect {
                source,
                target,
                strength,
            } => {
                assert_eq!(source, "fire");
                assert_eq!(target, "heat");
                assert!((strength - 0.9).abs() < 0.001);
            }
            _ => panic!("Expected BondConnect"),
        }
    }
}
