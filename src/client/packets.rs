//! BYOND Packet Framing
//!
//! Handles packet construction and parsing for the BYOND client protocol.

use anyhow::{anyhow, Result};
use std::io::{Read, Write};

/// Known packet types in the BYOND protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum PacketType {
    Quit = 0x0000,
    Handshake = 0x0001,
    RegisterVerb = 0x0011,
    AuthAck = 0x001a,
    AtomAppearance = 0x0026,
    Message = 0x0027,
    /// For unknown packet types we haven't mapped yet
    Unknown(u16),
}

impl From<u16> for PacketType {
    fn from(value: u16) -> Self {
        match value {
            0x0000 => PacketType::Quit,
            0x0001 => PacketType::Handshake,
            0x0011 => PacketType::RegisterVerb,
            0x001a => PacketType::AuthAck,
            0x0026 => PacketType::AtomAppearance,
            0x0027 => PacketType::Message,
            other => PacketType::Unknown(other),
        }
    }
}

impl From<PacketType> for u16 {
    fn from(pt: PacketType) -> u16 {
        match pt {
            PacketType::Quit => 0x0000,
            PacketType::Handshake => 0x0001,
            PacketType::RegisterVerb => 0x0011,
            PacketType::AuthAck => 0x001a,
            PacketType::AtomAppearance => 0x0026,
            PacketType::Message => 0x0027,
            PacketType::Unknown(v) => v,
        }
    }
}

/// A BYOND protocol packet
#[derive(Debug, Clone)]
pub struct Packet {
    /// Sequence number (0 for handshake packets)
    pub seq: u16,
    /// Packet type identifier
    pub packet_type: PacketType,
    /// Raw packet data (may be encrypted)
    pub data: Vec<u8>,
}

impl Packet {
    /// Create a new packet
    pub fn new(seq: u16, packet_type: PacketType, data: Vec<u8>) -> Self {
        Self {
            seq,
            packet_type,
            data,
        }
    }

    /// Read a packet from a stream
    ///
    /// Packet format (after handshake):
    /// - Bytes 0-1: Sequence number (big-endian u16)
    /// - Bytes 2-3: Packet type (big-endian u16)
    /// - Bytes 4-5: Data length (big-endian u16)
    /// - Bytes 6+: Data payload
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut header = [0u8; 6];
        reader.read_exact(&mut header)?;

        let seq = u16::from_be_bytes([header[0], header[1]]);
        let packet_type = u16::from_be_bytes([header[2], header[3]]);
        let length = u16::from_be_bytes([header[4], header[5]]) as usize;

        let mut data = vec![0u8; length];
        if length > 0 {
            reader.read_exact(&mut data)?;
        }

        Ok(Self {
            seq,
            packet_type: packet_type.into(),
            data,
        })
    }

    /// Write a packet to a stream
    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        let type_val: u16 = self.packet_type.into();
        let length = self.data.len() as u16;

        // Write header
        writer.write_all(&self.seq.to_be_bytes())?;
        writer.write_all(&type_val.to_be_bytes())?;
        writer.write_all(&length.to_be_bytes())?;

        // Write data
        if !self.data.is_empty() {
            writer.write_all(&self.data)?;
        }

        writer.flush()?;
        Ok(())
    }

    /// Serialize packet to bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let type_val: u16 = self.packet_type.into();
        let length = self.data.len() as u16;

        let mut bytes = Vec::with_capacity(6 + self.data.len());
        bytes.extend_from_slice(&self.seq.to_be_bytes());
        bytes.extend_from_slice(&type_val.to_be_bytes());
        bytes.extend_from_slice(&length.to_be_bytes());
        bytes.extend_from_slice(&self.data);

        bytes
    }
}

/// Advance the sequence number according to BYOND's algorithm
///
/// The sequence number is advanced using a specific formula:
/// seq = seq * 0x43d4
/// seq = seq + (seq / 0xFFF1) * 15
/// if seq == 0: seq = 1
pub fn advance_sequence(seq: u16) -> u16 {
    let mut s = seq as u32;
    s = s.wrapping_mul(0x43d4);
    s = s.wrapping_add((s / 0xFFF1) * 15);
    let result = (s & 0xFFFF) as u16;
    if result == 0 {
        1
    } else {
        result
    }
}

/// Build a client handshake packet
///
/// Handshake data format:
/// - Bytes 0-3: BYOND version (u32 LE)
/// - Bytes 4-7: Minimum version (u32 LE)
/// - Bytes 8-11: Encryption key component (u32 LE)
/// - Bytes 12-13: Initial sequence number (u16 LE)
/// - Additional padding/data may follow
pub fn build_handshake(byond_version: u32, min_version: u32, encryption_key: u32, initial_seq: u16) -> Packet {
    let mut data = Vec::with_capacity(32);

    // Version info (little-endian)
    data.extend_from_slice(&byond_version.to_le_bytes());
    data.extend_from_slice(&min_version.to_le_bytes());

    // Encryption key component
    data.extend_from_slice(&encryption_key.to_le_bytes());

    // Initial sequence number
    data.extend_from_slice(&initial_seq.to_le_bytes());

    // Padding (observed in captures - may need adjustment)
    data.extend_from_slice(&[0u8; 16]);

    Packet::new(0, PacketType::Handshake, data)
}

/// Parse encryption key addition from server handshake response
///
/// Server handshake contains an additional key component that must be
/// added to the client's encryption key.
pub fn parse_server_handshake_key(data: &[u8]) -> Result<u32> {
    if data.len() < 15 {
        return Err(anyhow!("Server handshake too short"));
    }

    // Skip to position 15 and scan for marker pattern
    let mut ptr = 15;

    while ptr + 4 <= data.len() {
        let v = u32::from_le_bytes([
            data[ptr],
            data.get(ptr + 1).copied().unwrap_or(0),
            data.get(ptr + 2).copied().unwrap_or(0),
            data.get(ptr + 3).copied().unwrap_or(0),
        ]);

        ptr += 4;

        // Check marker: (v + 0x71bd632f) & 0x04008000 == 0 means we found the key
        let marker = v.wrapping_add(0x71bd632f) & 0x04008000;
        if marker == 0 {
            break;
        }
    }

    // Read the key addition value
    if ptr + 4 > data.len() {
        return Err(anyhow!("Could not find key addition in server handshake"));
    }

    let add_to_key = u32::from_le_bytes([
        data[ptr],
        data[ptr + 1],
        data[ptr + 2],
        data[ptr + 3],
    ]);

    Ok(add_to_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sequence_advancement() {
        // Test basic advancement
        let seq1 = advance_sequence(1);
        assert_ne!(seq1, 1);
        assert_ne!(seq1, 0);

        // Test that 0 is avoided
        let mut seq = 1u16;
        for _ in 0..10000 {
            seq = advance_sequence(seq);
            assert_ne!(seq, 0, "Sequence should never be 0");
        }
    }

    #[test]
    fn test_packet_roundtrip() {
        let packet = Packet::new(
            0x1234,
            PacketType::Message,
            b"test data".to_vec(),
        );

        let bytes = packet.to_bytes();
        let mut cursor = std::io::Cursor::new(bytes);
        let parsed = Packet::read_from(&mut cursor).unwrap();

        assert_eq!(parsed.seq, 0x1234);
        assert_eq!(parsed.packet_type, PacketType::Message);
        assert_eq!(parsed.data, b"test data");
    }

    #[test]
    fn test_packet_type_conversion() {
        assert_eq!(PacketType::from(0x0001), PacketType::Handshake);
        assert_eq!(PacketType::from(0x0026), PacketType::AtomAppearance);
        assert_eq!(PacketType::from(0xFFFF), PacketType::Unknown(0xFFFF));

        let type_val: u16 = PacketType::Handshake.into();
        assert_eq!(type_val, 0x0001);
    }
}
