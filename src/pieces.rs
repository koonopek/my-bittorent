use std::cmp;

use crate::{
    messaging::{read_message, send_message, MessageType},
    peers::{handshake, PeerConnection},
    sha1_it, MetaInfoFile,
};

pub fn download_piece(peer: &str, info: &MetaInfoFile, piece_index: usize) -> (usize, Vec<u8>) {
    let mut connection = handshake(peer, &info);

    assert_eq!(
        read_message(&mut connection).message_type,
        MessageType::BitField
    );

    send_message(&mut connection, MessageType::Intrested, vec![]);

    assert_eq!(
        read_message(&mut connection).message_type,
        MessageType::Unchoked
    );

    let mut chunks_read = 0;

    let length_to_read = cmp::min(
        info.length - (piece_index * info.piece_length),
        info.piece_length,
    );

    loop {
        let current_chunk_to_read: i64 = length_to_read as i64 - (16 * 1024 * chunks_read) as i64;
        match current_chunk_to_read {
            x if x <= 0 => break,
            x if x >= 16 * 1024 => request_piece_part(
                &mut connection,
                piece_index as u32,
                chunks_read as u32,
                16 * 1024,
            ),
            x => request_piece_part(
                &mut connection,
                piece_index as u32,
                chunks_read as u32,
                x as u32,
            ),
        }
        chunks_read += 1;
    }

    let mut piece = Vec::with_capacity(info.piece_length);
    for _ in 0..chunks_read {
        let message = read_message(&mut connection);
        if message.message_type == MessageType::Piece {
            piece.extend_from_slice(&message.payload[8..])
        }
    }

    println!(
        "Checking piece hash {} == {}",
        info.piece_hashes[piece_index],
        hex::encode(sha1_it(&piece))
    );

    assert_eq!(info.piece_hashes[piece_index], hex::encode(sha1_it(&piece)));

    connection
        .tcp_stream
        .shutdown(std::net::Shutdown::Both)
        .expect("Failed to close tcp stream");

    (piece_index, piece)
}

pub fn request_piece_part(
    connection: &mut PeerConnection,
    piece_index: u32,
    offset_block: u32,
    bytes_to_read: u32,
) {
    let begin: u32 = offset_block * 16 * 1024;
    let mut payload = Vec::with_capacity(12);
    payload.extend_from_slice(&piece_index.to_be_bytes());
    payload.extend_from_slice(&begin.to_be_bytes());
    payload.extend_from_slice(&bytes_to_read.to_be_bytes());
    send_message(connection, MessageType::Request, payload);
}
