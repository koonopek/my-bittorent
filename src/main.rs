use std::{
    env,
    io::{IoSlice, Read, Write},
};

use bittorrent_starter_rust::{
    decode_bencoded_value, discover_peers, get_metafile_info, handshake,
};
use serde_json::json;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    let file_path = &args[2];

    if command == "decode" {
        // Uncomment this block to pass the first stage
        let mut encoded_value = args[2].bytes().into_iter();
        let decoded_value = decode_bencoded_value(&mut encoded_value).unwrap();
        println!("{}", json!(decoded_value));
    } else if command == "info" {
        let info = get_metafile_info(file_path);
        print!("{}", info);
    } else if command == "peers" {
        let info = get_metafile_info(file_path);
        let peers = discover_peers(&info);
        println!("{:?}", peers);
    } else if command == "handshake" {
        let info = get_metafile_info(file_path);
        let peer = &args[3];

        let connection = handshake(peer, &info);
        println!("Handshaked with Peer ID: {}", connection.peer_id);
    } else if command == "download_piece" {
        let (param_name, save_to, torrent_info_path, piece_number) =
            (&args[2], &args[3], &args[4], &args[5]);

        let info = get_metafile_info(torrent_info_path);
        let peers = discover_peers(&info);
        let peer = peers.get(0).expect("Expected at least one peer");

        let mut connection = handshake(peer, &info);
        println!("sucessful handshake");

        read_message(&mut connection);

        // send instrested
        send_message(&mut connection, MessageType::Intrested, vec![]);
        println!("Sent intrested message");

        read_message(&mut connection);

        let full_pieces_count: u32 = info.piece_length as u32 / (16 * 1024);
        println!("Full pieces to read {}", full_pieces_count);

        let piece_index: u32 = piece_number.parse().expect("Failed to parse piece index");

        for piece_i in 0..full_pieces_count {
            request_piece_part(&mut connection, piece_index, piece_i);
        }

        let last_piece_begin: u32 = full_pieces_count * 16 * 1024;
        let last_piece_length: u32 = info.piece_length as u32 - last_piece_begin;
        let need_last_piece = last_piece_length > 0;
        if need_last_piece {
            request_piece_part(&mut connection, piece_index, full_pieces_count);
        }

        for _ in 0..full_pieces_count {
            read_message(&mut connection);
        }

        if need_last_piece {
            read_message(&mut connection);
        }
    } else {
        println!("unknown command: {}", args[1])
    }
}

fn request_piece_part(
    connection: &mut bittorrent_starter_rust::PeerConnection,
    piece_index: u32,
    offset_block: u32,
) {
    let begin: u32 = offset_block * 16 * 1024;
    let length: u32 = 16 * 1024;

    let mut payload = Vec::with_capacity(12);
    payload.extend_from_slice(&piece_index.to_be_bytes());
    payload.extend_from_slice(&begin.to_be_bytes());
    payload.extend_from_slice(&length.to_be_bytes());

    send_message(connection, MessageType::Request, payload);
}

fn send_message(
    connection: &mut bittorrent_starter_rust::PeerConnection,
    message_type: MessageType,
    payload: Vec<u8>,
) {
    let payload_len = payload.len() + 1;

    let mut message_payload: Vec<u8> = Vec::with_capacity(4 + payload_len);

    message_payload.extend_from_slice(&payload_len.to_be_bytes());
    message_payload.push(message_type as u8);
    message_payload.extend(payload);

    connection
        .tcp_stream
        .write_all(&message_payload)
        .expect("Failed to write to tcp stream");
}

fn read_message(connection: &mut bittorrent_starter_rust::PeerConnection) {
    let mut payload_size_buf: [u8; 4] = [0; 4];
    connection
        .tcp_stream
        .read_exact(&mut payload_size_buf)
        .expect("failed to reade message size");

    println!("Reading new message");

    let mut message_id_buf: [u8; 1] = [0; 1];
    connection
        .tcp_stream
        .read_exact(&mut message_id_buf)
        .expect("Failed to read message id");

    let message_type = match message_id_buf[0] {
        1 => MessageType::Unchoked,
        5 => MessageType::BitField,
        7 => MessageType::Piece,
        id => panic!("Unknown message type {}", id),
    };

    println!(">>Message type: {:?}", message_type);

    let payload_size = match u32::from_be_bytes(payload_size_buf) {
        x if x == 0 => 0 as usize,
        x => (x - 1) as usize,
    };

    println!(">>Payload size: {:?}", payload_size);

    let mut payload = vec![0; payload_size];

    connection
        .tcp_stream
        .read_exact(&mut payload)
        .expect("Failed to read buffer");

    println!("Finished reading message");
}

#[derive(Debug)]
enum MessageType {
    Unchoked = 1,
    Intrested = 2,
    BitField = 5,
    Request = 6,
    Piece = 7,
}
