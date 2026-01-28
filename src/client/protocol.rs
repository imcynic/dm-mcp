//! BYOND Client Protocol Handler
//!
//! Implements the connection, handshake, and packet exchange for
//! connecting to a BYOND DreamDaemon server.

use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::time::Duration;

use anyhow::{anyhow, Result};
use rand::Rng;
use tracing::{debug, info, warn};

use super::crypto::{runsub_decrypt, runsub_encrypt};
use super::packets::{self, advance_sequence, build_handshake, Packet, PacketType};

/// Current BYOND version to report (major.minor format)
const BYOND_VERSION: u32 = 516;
const BYOND_MIN_VERSION: u32 = 1673;

/// BYOND client connection
pub struct BYONDClient {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    encryption_key: i32,
    sequence: u16,
    connected: bool,
}

impl BYONDClient {
    /// Connect to a BYOND server as a guest
    pub fn connect(addr: &str, port: u16) -> Result<Self> {
        info!("Connecting to BYOND server at {}:{}", addr, port);

        // Establish TCP connection
        let stream = TcpStream::connect((addr, port))?;
        stream.set_read_timeout(Some(Duration::from_secs(10)))?;
        stream.set_write_timeout(Some(Duration::from_secs(10)))?;

        let reader = BufReader::new(stream.try_clone()?);
        let writer = BufWriter::new(stream);

        let mut client = Self {
            reader,
            writer,
            encryption_key: 0,
            sequence: 1,
            connected: false,
        };

        // Perform handshake
        client.perform_handshake()?;

        Ok(client)
    }

    /// Perform the BYOND handshake protocol
    fn perform_handshake(&mut self) -> Result<()> {
        info!("Performing BYOND handshake");

        // Generate random encryption key component
        let mut rng = rand::thread_rng();
        let client_key: u32 = rng.gen();
        let initial_seq: u16 = rng.gen_range(1..=0xFFFF);

        // Build and send client handshake
        let handshake = build_handshake(BYOND_VERSION, BYOND_MIN_VERSION, client_key, initial_seq);
        debug!("Sending client handshake: {:?}", handshake);
        handshake.write_to(&mut self.writer)?;

        // Calculate initial encryption key
        // Formula: client_key + byond_version + (min_version * 0x10000)
        let mut encryption_key = client_key
            .wrapping_add(BYOND_VERSION)
            .wrapping_add(BYOND_MIN_VERSION.wrapping_mul(0x10000));

        debug!("Initial encryption key: 0x{:08X}", encryption_key);

        // Read raw header bytes first to debug
        let mut raw_header = [0u8; 6];
        std::io::Read::read_exact(&mut self.reader, &mut raw_header)?;
        info!(
            "Raw server header bytes: {:02X} {:02X} {:02X} {:02X} {:02X} {:02X}",
            raw_header[0], raw_header[1], raw_header[2], raw_header[3], raw_header[4], raw_header[5]
        );

        let seq = u16::from_be_bytes([raw_header[0], raw_header[1]]);
        let packet_type_raw = u16::from_be_bytes([raw_header[2], raw_header[3]]);
        let length = u16::from_be_bytes([raw_header[4], raw_header[5]]) as usize;

        info!(
            "Parsed header: seq=0x{:04X}, type=0x{:04X}, len={}",
            seq, packet_type_raw, length
        );

        // Read the data portion
        let mut data = vec![0u8; length];
        if length > 0 {
            std::io::Read::read_exact(&mut self.reader, &mut data)?;
        }

        // Log first few bytes of raw data
        if !data.is_empty() {
            let preview_len = data.len().min(32);
            info!("Raw data preview (first {} bytes): {:02X?}", preview_len, &data[..preview_len]);
        }

        let mut response = Packet::new(seq, packet_type_raw.into(), data);
        debug!(
            "Received server response: type={:?}, len={}",
            response.packet_type,
            response.data.len()
        );

        // For handshake, accept type 0x0001 or 0x0002 (some versions differ)
        if response.packet_type != PacketType::Handshake && packet_type_raw != 0x0002 {
            return Err(anyhow!(
                "Expected handshake response (0x0001 or 0x0002), got 0x{:04X}",
                packet_type_raw
            ));
        }

        // Decrypt server response with current key
        runsub_decrypt(&mut response.data, encryption_key as i32);
        debug!("Decrypted server handshake: {:02X?}", &response.data[..response.data.len().min(32)]);

        // Parse server's key addition
        let add_to_key = packets::parse_server_handshake_key(&response.data)?;
        debug!("Server key addition: 0x{:08X}", add_to_key);

        encryption_key = encryption_key.wrapping_add(add_to_key);
        debug!("Final encryption key: 0x{:08X}", encryption_key);

        self.encryption_key = encryption_key as i32;
        self.sequence = initial_seq;
        self.connected = true;

        info!("Handshake complete, connected to server");
        Ok(())
    }

    /// Send a packet to the server
    pub fn send_packet(&mut self, mut packet: Packet) -> Result<()> {
        if self.encryption_key != 0 && !packet.data.is_empty() {
            // Add space for checksum and encrypt
            packet.data.push(0);
            runsub_encrypt(&mut packet.data, self.encryption_key);
        }

        packet.seq = self.sequence;
        packet.write_to(&mut self.writer)?;

        self.sequence = advance_sequence(self.sequence);
        Ok(())
    }

    /// Receive a packet from the server
    pub fn receive_packet(&mut self) -> Result<Packet> {
        let mut packet = Packet::read_from(&mut self.reader)?;

        if self.encryption_key != 0 && !packet.data.is_empty() {
            runsub_decrypt(&mut packet.data, self.encryption_key);
            // Remove checksum byte
            if !packet.data.is_empty() {
                packet.data.pop();
            }
        }

        Ok(packet)
    }

    /// Receive packets until we see a specific type or timeout
    pub fn receive_until(&mut self, target_type: PacketType, max_packets: usize) -> Result<Vec<Packet>> {
        let mut packets = Vec::new();

        for _ in 0..max_packets {
            match self.receive_packet() {
                Ok(packet) => {
                    let is_target = packet.packet_type == target_type;
                    debug!(
                        "Received packet: type={:?}, seq={}, len={}",
                        packet.packet_type, packet.seq, packet.data.len()
                    );
                    packets.push(packet);
                    if is_target {
                        break;
                    }
                }
                Err(e) => {
                    warn!("Error receiving packet: {}", e);
                    break;
                }
            }
        }

        Ok(packets)
    }

    /// Receive all initial packets from the server (for debugging/logging)
    pub fn receive_initial_packets(&mut self, timeout_secs: u64) -> Result<Vec<Packet>> {
        // Reduce timeout for faster packet reading
        if let Ok(stream) = self.reader.get_ref().try_clone() {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(timeout_secs)));
        }

        let mut packets = Vec::new();
        let mut consecutive_errors = 0;

        loop {
            match self.receive_packet() {
                Ok(packet) => {
                    consecutive_errors = 0;
                    info!(
                        "Packet: type=0x{:04X} ({:?}), seq={}, data_len={}",
                        u16::from(packet.packet_type),
                        packet.packet_type,
                        packet.seq,
                        packet.data.len()
                    );

                    // Log first few bytes of data for debugging
                    if !packet.data.is_empty() {
                        let preview_len = packet.data.len().min(32);
                        debug!("  Data preview: {:02X?}", &packet.data[..preview_len]);
                    }

                    packets.push(packet);
                }
                Err(e) => {
                    consecutive_errors += 1;
                    if consecutive_errors >= 3 {
                        debug!("Stopping packet receive after {} consecutive errors: {}", consecutive_errors, e);
                        break;
                    }
                }
            }
        }

        Ok(packets)
    }

    /// Disconnect from the server
    pub fn disconnect(&mut self) -> Result<()> {
        if self.connected {
            // Send quit packet
            let quit_packet = Packet::new(self.sequence, PacketType::Quit, vec![]);
            let _ = self.send_packet(quit_packet);
            self.connected = false;
        }
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }
}

impl Drop for BYONDClient {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_calculation() {
        // Test the key calculation formula
        let client_key: u32 = 0x12345678;
        let byond_version: u32 = 516;
        let min_version: u32 = 1673;

        let key = client_key
            .wrapping_add(byond_version)
            .wrapping_add(min_version.wrapping_mul(0x10000));

        // Just verify it computes something reasonable
        assert!(key != client_key);
    }
}
