pub mod utils;

use utils::binary::{BinaryError, BinaryReader, BinaryWritter, Result};
use bytes::{BytesMut};
use uuid::Uuid;

fn main() -> Result<()> {
    println!("--- Basic Write/Read Example ---");
    example_basic_write_read()?;

    println!("\n--- Endianness Example ---");
    example_endianness()?;

    println!("\n--- VarInts & ZigZag Example ---");
    example_varints()?;

    println!("\n--- Strings & Bytes Example ---");
    example_strings_bytes()?;

    println!("\n--- UUID Example ---");
    example_uuid()?;

    println!("\n--- Error Handling (EOF) Example ---");
    example_error_handling(); // Doesn't return Result, prints error

    println!("\n--- Combined 'Packet' Simulation ---");
    example_packet_simulation()?;

    Ok(()) // Indicate success
}

// --- Example Functions ---

fn example_basic_write_read() -> Result<()> {
    let mut writer = BytesMut::new();

    // Write some basic data
    writer.write_u8(0x42)?; // Write a byte
    writer.write_i32_le(-1000)?; // Write a little-endian i32
    writer.write_bool(true)?; // Write a boolean (as 0x01)

    println!("Written buffer (hex): {:02X?}", writer.as_ref());

    // Freeze the buffer for reading (simulates receiving data)
    let mut reader = writer.freeze();

    // Read the data back
    let byte_val = reader.read_u8()?;
    let int_val = reader.read_i32_le()?;
    let bool_val = reader.read_bool()?;

    println!("Read u8: {}", byte_val);
    println!("Read i32_le: {}", int_val);
    println!("Read bool: {}", bool_val);

    // Assertions
    assert_eq!(byte_val, 0x42);
    assert_eq!(int_val, -1000);
    assert!(bool_val);

    // Check if buffer is fully read
    assert!(reader.is_empty(), "Reader should be empty after reading all data");

    Ok(())
}

fn example_endianness() -> Result<()> {
    let mut writer = BytesMut::new();
    let value: u16 = 0xABCD;

    writer.write_u16_le(value)?; // CD AB
    writer.write_u16_be(value)?; // AB CD

    println!("Written buffer (hex): {:02X?}", writer.as_ref());

    let mut reader = writer.freeze();

    let le_val = reader.read_u16_le()?;
    let be_val = reader.read_u16_be()?;

    println!("Read u16_le: {:#06X}", le_val);
    println!("Read u16_be: {:#06X}", be_val);

    assert_eq!(le_val, value);
    assert_eq!(be_val, value);
    assert!(reader.is_empty());

    Ok(())
}

fn example_varints() -> Result<()> {
    let mut writer = BytesMut::new();

    let val_u32: u32 = 2097151; // Requires 3 bytes
    let val_i32: i32 = -16384; // ZigZag requires 3 bytes
    let val_u64: u64 = u64::MAX; // Requires 10 bytes

    writer.write_varu32(val_u32)?;
    writer.write_vari32(val_i32)?;
    writer.write_varu64(val_u64)?;

    println!("Written buffer (hex, VarInts): {:02X?}", writer.as_ref());

    let mut reader = writer.freeze();

    let read_u32 = reader.read_varu32()?;
    let read_i32 = reader.read_vari32()?;
    let read_u64 = reader.read_varu64()?;

    println!("Read varu32: {}", read_u32);
    println!("Read vari32: {}", read_i32);
    println!("Read varu64: {}", read_u64);

    assert_eq!(read_u32, val_u32);
    assert_eq!(read_i32, val_i32);
    assert_eq!(read_u64, val_u64);
    assert!(reader.is_empty());

    Ok(())
}

fn example_strings_bytes() -> Result<()> {
    let mut writer = BytesMut::new();
    let message = "Amethyst Server Rocks! 🚀";
    let raw_data: &[u8] = &[0x01, 0x02, 0x03, 0x04, 0x05];

    writer.write_string_varint_len(message)?;
    writer.write_bytes_varint_len(raw_data)?;

    println!("Written buffer (hex, prefixed string/bytes): {:02X?}", writer.as_ref());

    let mut reader = writer.freeze();

    let read_message = reader.read_string_varint_len()?;
    let read_bytes = reader.read_bytes_varint_len()?; // Returns a Bytes object

    println!("Read string: '{}'", read_message);
    println!("Read bytes: {:02X?}", read_bytes.as_ref());

    assert_eq!(read_message, message);
    assert_eq!(read_bytes.as_ref(), raw_data); // Compare slices
    assert!(reader.is_empty());

    Ok(())
}

fn example_uuid() -> Result<()> {
    let mut writer = BytesMut::new();
    // Example UUID
    let original_uuid = Uuid::parse_str("f81d4fae-7dec-11d0-a765-00a0c91e6bf6").unwrap();

    // MCBE often uses Little Endian for UUIDs in packets
    writer.write_uuid_le(&original_uuid)?;

    println!("Written UUID LE buffer (hex): {:02X?}", writer.as_ref());

    let mut reader = writer.freeze();

    let read_uuid = reader.read_uuid_le()?;

    println!("Original UUID: {}", original_uuid);
    println!("Read UUID LE:  {}", read_uuid);

    assert_eq!(read_uuid, original_uuid);
    assert!(reader.is_empty());

    Ok(())
}

fn example_error_handling() {
    let mut writer = BytesMut::new();
    writer.write_u8(0xFF).unwrap(); // Write only 1 byte

    let mut reader = writer.freeze();

    // Read the first byte - should succeed
    match reader.read_u8() {
        Ok(val) => println!("Successfully read first byte: {:#04X}", val),
        Err(e) => println!("Unexpected error reading first byte: {}", e), // Should not happen
    }

    // Attempt to read another byte - should fail with EOF
    match reader.read_u8() {
        Ok(_) => println!("Error: Unexpectedly read second byte!"),
        Err(BinaryError::UnexpectedEof { needed, remaining }) => {
            println!(
                "Correctly caught EOF error: needed {} bytes, but only {} remaining.",
                needed, remaining
            );
        }
        Err(e) => println!("Caught unexpected error type: {}", e),
    }

    // Example of reading VarInt past EOF
    let mut writer_varint = BytesMut::new();
    writer_varint.write_bytes(&[0x80, 0x80]).unwrap(); // Incomplete VarInt
    let mut reader_varint = writer_varint.freeze();

    match reader_varint.read_varu32() {
        Ok(_) => println!("Error: Unexpectedly read incomplete VarInt!"),
        Err(BinaryError::UnexpectedEof { .. }) => {
            println!("Correctly caught EOF error while reading VarInt.");
        }
        Err(e) => println!("Caught unexpected error type reading VarInt: {}", e),
    }
}

fn example_packet_simulation() -> Result<()> {
    // --- Simulate Writing a Login Packet ---
    let mut writer = BytesMut::new();

    let packet_id: u8 = 0x01; // Hypothetical Login Packet ID
    let username = "Player123";
    let protocol_version: i32 = 622; // Example protocol version
    let client_uuid = Uuid::new_v4();

    writer.write_u8(packet_id)?;
    writer.write_string_varint_len(username)?;
    writer.write_vari32(protocol_version)?; // Using signed VarInt for protocol
    writer.write_uuid_le(&client_uuid)?; // Little Endian UUID

    println!("Simulated Login Packet buffer (hex): {:02X?}", writer.as_ref());
    let packet_bytes = writer.freeze(); // Get the final Bytes

    // --- Simulate Reading the Login Packet ---
    let mut reader = packet_bytes; // Use the frozen Bytes

    let read_id = reader.read_u8()?;
    let read_username = reader.read_string_varint_len()?;
    let read_protocol = reader.read_vari32()?;
    let read_uuid = reader.read_uuid_le()?;

    println!("--- Read Simulated Login Packet ---");
    println!("Packet ID: {:#04X}", read_id);
    println!("Username: {}", read_username);
    println!("Protocol Version: {}", read_protocol);
    println!("Client UUID: {}", read_uuid);

    // Assertions
    assert_eq!(read_id, packet_id);
    assert_eq!(read_username, username);
    assert_eq!(read_protocol, protocol_version);
    assert_eq!(read_uuid, client_uuid);
    assert!(reader.is_empty(), "Reader should be empty after reading packet");

    Ok(())
}