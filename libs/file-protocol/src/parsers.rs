//
// Copyright (C) 2018 Kubos Corporation
//
// Licensed under the Apache License, Version 2.0 (the "License")
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

use super::Message;
use serde_cbor::Value;

pub fn parse_message(message: Value) -> Result<Message, String> {
    if let Some(msg) = parse_export_request(message.to_owned())? {
        return Ok(msg);
    }
    if let Some(msg) = parse_import_request(message.to_owned())? {
        return Ok(msg);
    }
    if let Some(msg) = parse_success_receive(message.to_owned())? {
        return Ok(msg);
    }
    if let Some(msg) = parse_success_transmit(message.to_owned())? {
        return Ok(msg);
    }
    if let Some(msg) = parse_bad_op(message.to_owned())? {
        return Ok(msg);
    }
    if let Some(msg) = parse_ack(message.to_owned())? {
        return Ok(msg);
    }
    if let Some(msg) = parse_nak(message.to_owned())? {
        return Ok(msg);
    }
    if let Some(msg) = parse_chunk(message.to_owned())? {
        return Ok(msg);
    }
    if let Some(msg) = parse_sync(message.to_owned())? {
        return Ok(msg);
    }

    return Err("No message found".to_string());
}

// Parse out export request
// { channel_id, "export", hash, path, [, mode] }
pub fn parse_export_request(message: Value) -> Result<Option<Message>, String> {
    let data = match message {
        Value::Array(val) => val.to_owned(),
        _ => return Err("Unable to parse message: Data not an array".to_owned()),
    };
    let mut pieces = data.iter();

    let first_param: Value = pieces
        .next()
        .ok_or(format!("Unable to parse message: No contents"))?
        .to_owned();

    if let Value::U64(channel_id) = first_param {
        if let Value::String(op) = pieces.next().ok_or("".to_owned()).unwrap() {
            if op == "export" {
                let hash = match pieces
                    .next()
                    .ok_or(format!("Unable to parse export message: No hash param"))?
                {
                    Value::String(val) => val,
                    _ => {
                        return Err("Unable to parse export message: Invalid hash param".to_owned())
                    }
                };

                let path = match pieces
                    .next()
                    .ok_or(format!("Unable to parse export message: No path param"))?
                {
                    Value::String(val) => val,
                    _ => {
                        return Err("Unable to parse export message: Invalid path param".to_owned())
                    }
                };

                let mode = match pieces.next() {
                    Some(Value::U64(num)) => Some(*num as u32),
                    _ => None,
                };

                return Ok(Some(Message::ReqReceive(
                    channel_id,
                    hash.to_owned(),
                    path.to_owned(),
                    mode,
                )));
            }
        }
    }

    return Ok(None);
}

// Parse out import request
// { channel_id, "import", path }
pub fn parse_import_request(message: Value) -> Result<Option<Message>, String> {
    let data = match message {
        Value::Array(val) => val.to_owned(),
        _ => return Err("Unable to parse message: Data not an array".to_owned()),
    };
    let mut pieces = data.iter();

    let first_param: Value = pieces
        .next()
        .ok_or(format!("Unable to parse message: No contents"))?
        .to_owned();

    if let Value::U64(channel_id) = first_param {
        if let Value::String(op) = pieces.next().ok_or("".to_owned()).unwrap() {
            if op == "import" {
                let path = match pieces
                    .next()
                    .ok_or(format!("Unable to parse import message: No path param"))?
                {
                    Value::String(val) => val,
                    _ => {
                        return Err("Unable to parse import message: Invalid path param".to_owned())
                    }
                };
                return Ok(Some(Message::ReqTransmit(channel_id, path.to_owned())));
            }
        }
    }

    return Ok(None);
}

// Parse out success received message
// { channel_id, true }
pub fn parse_success_receive(message: Value) -> Result<Option<Message>, String> {
    let data = match message {
        Value::Array(val) => val.to_owned(),
        _ => return Err("Unable to parse message: Data not an array".to_owned()),
    };
    let mut pieces = data.iter();

    let first_param: Value = pieces
        .next()
        .ok_or(format!("Unable to parse message: No contents"))?
        .to_owned();

    if let Value::U64(channel_id) = first_param {
        if let Value::Bool(result) = pieces.next().ok_or("".to_owned()).unwrap() {
            if *result == true {
                // Good - { channel_id, true, ...values }
                if let None = pieces.next() {
                    return Ok(Some(Message::SuccessReceive(channel_id)));
                }
            }
        }
    }

    return Ok(None);
}

// Parse out success transmit message
// { channel_id, "true", ..values }
pub fn parse_success_transmit(message: Value) -> Result<Option<Message>, String> {
    let data = match message {
        Value::Array(val) => val.to_owned(),
        _ => return Err("Unable to parse message: Data not an array".to_owned()),
    };
    let mut pieces = data.iter();

    let first_param: Value = pieces
        .next()
        .ok_or(format!("Unable to parse message: No contents"))?
        .to_owned();

    if let Value::U64(channel_id) = first_param {
        if let Value::Bool(result) = pieces.next().ok_or("".to_owned()).unwrap() {
            if *result == true {
                // Good - { channel_id, true, ...values }
                if let Some(piece) = pieces.next() {
                    // It's a good result after an 'import' operation
                    let hash = match piece {
                        Value::String(val) => val,
                        _ => {
                            return Err(
                                "Unable to parse success message: Invalid hash param".to_owned()
                            )
                        }
                    };

                    let num_chunks = match pieces.next().ok_or(format!(
                        "Unable to parse success message: No num_chunks param"
                    ))? {
                        Value::U64(val) => *val,
                        _ => {
                            return Err("Unable to parse success message: Invalid num_chunks param"
                                .to_owned())
                        }
                    };

                    let mode = match pieces.next() {
                        Some(Value::U64(val)) => Some(*val as u32),
                        _ => None,
                    };

                    // Return the file info
                    return Ok(Some(Message::SuccessTransmit(
                        channel_id,
                        hash.to_string(),
                        num_chunks as u32,
                        mode,
                    )));
                }
            }
        }
    }

    return Ok(None);
}

// Parse out bad
// { channel_id, "false", ..values }
pub fn parse_bad_op(message: Value) -> Result<Option<Message>, String> {
    let data = match message {
        Value::Array(val) => val.to_owned(),
        _ => return Err("Unable to parse message: Data not an array".to_owned()),
    };
    let mut pieces = data.iter();

    let first_param: Value = pieces
        .next()
        .ok_or(format!("Unable to parse message: No contents"))?
        .to_owned();

    if let Value::U64(channel_id) = first_param {
        if let Value::Bool(result) = pieces.next().ok_or("".to_owned()).unwrap() {
            if *result == false {
                let error = match pieces
                    .next()
                    .ok_or(format!("Unable to parse failure message: No error param"))?
                {
                    Value::String(val) => val,
                    _ => {
                        return Err("Unable to parse failure message: Invalid error param".to_owned())
                    }
                };

                return Ok(Some(Message::Failure(channel_id, error.to_owned())));
            }
        }
    }

    return Ok(None);
}

// Parse out ack
// { hash, true, num_chunks }
pub fn parse_ack(message: Value) -> Result<Option<Message>, String> {
    let data = match message {
        Value::Array(val) => val.to_owned(),
        _ => return Err("Unable to parse message: Data not an array".to_owned()),
    };
    let mut pieces = data.iter();

    let first_param: Value = pieces
        .next()
        .ok_or(format!("Unable to parse message: No contents"))?
        .to_owned();

    if let Value::String(hash) = first_param {
        if let Value::Bool(true) = pieces.next().ok_or("".to_owned()).unwrap() {
            // It's an ACK: { hash, true, num_chunks }
            // Our data transfer (export) completed succesfully
            // self.stop_push(&hash)?;

            //TODO: Do something with the third param? (num_chunks)
            // Doesn't look like we do anything with num_chunks
            return Ok(Some(Message::ACK(hash)));
        }
    }

    return Ok(None);
}

// Parse out nak
// { hash, false, ..missing_chunks }
pub fn parse_nak(message: Value) -> Result<Option<Message>, String> {
    let data = match message {
        Value::Array(val) => val.to_owned(),
        _ => return Err("Unable to parse message: Data not an array".to_owned()),
    };
    let mut pieces = data.iter();

    let first_param: Value = pieces
        .next()
        .ok_or(format!("Unable to parse message: No contents"))?
        .to_owned();

    if let Value::String(hash) = first_param {
        if let Value::Bool(false) = pieces.next().ok_or("".to_owned()).unwrap() {
            let mut remaining_chunks: Vec<(u32, u32)> = vec![];
            let mut chunk_nums: Vec<u32> = vec![];
            for entry in pieces {
                if let Value::U64(chunk_num) = entry {
                    chunk_nums.push(*chunk_num as u32);
                }
            }

            for chunk in chunk_nums.chunks(2) {
                let first = chunk[0];
                let last = chunk[1];
                remaining_chunks.push((first, last));
            }

            return Ok(Some(Message::NAK(hash, Some(remaining_chunks))));
        }
    }

    return Ok(None);
}

// Parse out chunk
// { hash, chunk_index, data }
pub fn parse_chunk(message: Value) -> Result<Option<Message>, String> {
    let data = match message {
        Value::Array(val) => val.to_owned(),
        _ => return Err("Unable to parse message: Data not an array".to_owned()),
    };
    let mut pieces = data.iter();

    let first_param: Value = pieces
        .next()
        .ok_or(format!("Unable to parse message: No contents"))?
        .to_owned();

    if let Value::String(hash) = first_param {
        if let Value::U64(num) = pieces.next().ok_or("".to_owned()).unwrap() {
            if let Some(third_param) = pieces.next() {
                if let Value::Bytes(data) = third_param {
                    return Ok(Some(Message::ReceiveChunk(
                        hash,
                        *num as u32,
                        data.to_vec(),
                    )));
                } else {
                    return Err(format!(
                        "Unable to parse chunk message: Invalid data format"
                    ));
                }
            }
        }
    }

    return Ok(None);
}

// Parse out sync
// { hash, num_chunks }
// or
// { hash }
pub fn parse_sync(message: Value) -> Result<Option<Message>, String> {
    let data = match message {
        Value::Array(val) => val.to_owned(),
        _ => return Err("Unable to parse message: Data not an array".to_owned()),
    };
    let mut pieces = data.iter();

    let first_param: Value = pieces
        .next()
        .ok_or(format!("Unable to parse message: No contents"))?
        .to_owned();

    if let Value::String(hash) = first_param {
        if let Some(second_param) = pieces.next() {
            if let Value::U64(num) = second_param {
                if let None = pieces.next() {
                    // It's a sync message: { hash, num_chunks }
                    // TODO: Whoever processes this message should do the sync_and_send
                    //self.sync_and_send(&hash, Some(*num as u32));
                    return Ok(Some(Message::Metadata(hash, *num as u32)));
                }
            }
        } else {
            // It's a sync message: { hash }
            // TODO: Whoever processes this message should do the sync_and_send
            //self.sync_and_send(&hash, None)?;
            return Ok(Some(Message::Sync(hash)));
        }
    }

    return Ok(None);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_cbor::error::Error;
    use serde_cbor::{self, de, ser, Value};

    #[test]
    fn parse_export_request_good() {
        let data = ser::to_vec_packed(&(
            100,
            "export",
            "abcdefg".to_string(),
            "/target/path".to_string(),
            Some(0o600),
        )).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_export_request(message),
            Ok(Some(Message::ReqReceive(
                100,
                "abcdefg".to_string(),
                "/target/path".to_string(),
                Some(0o600)
            )))
        );
    }

    #[test]
    fn parse_export_request_no_hash() {
        let data = ser::to_vec_packed(&(100, "export")).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_export_request(message),
            Err("Unable to parse export message: No hash param".to_string())
        );
    }

    #[test]
    fn parse_import_request_good() {
        let data = ser::to_vec_packed(&(100, "import", "/import/path".to_string())).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_import_request(message),
            Ok(Some(Message::ReqTransmit(100, "/import/path".to_string(),)))
        );
    }

    #[test]
    fn parse_import_request_bad_path() {
        let data = ser::to_vec_packed(&(100, "import", 200)).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_import_request(message),
            Err("Unable to parse import message: Invalid path param".to_string())
        );
    }

    #[test]
    fn parse_success_receive_good() {
        let data = ser::to_vec_packed(&(100, true)).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_success_receive(message),
            Ok(Some(Message::SuccessReceive(100)))
        );
    }

    #[test]
    fn parse_success_transmit_good() {
        let data = ser::to_vec_packed(&(100, true, "abcd".to_string(), 10, Some(0o00))).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_success_transmit(message),
            Ok(Some(Message::SuccessTransmit(
                100,
                "abcd".to_string(),
                10,
                Some(0o00)
            )))
        );
    }

    #[test]
    fn parse_bad_op_good() {
        let data = ser::to_vec_packed(&(100, false, "failed".to_string())).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_bad_op(message),
            Ok(Some(Message::Failure(100, "failed".to_string(),)))
        );
    }

    #[test]
    fn parse_ack_good() {
        let data = ser::to_vec_packed(&("abcd".to_string(), true)).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_ack(message),
            Ok(Some(Message::ACK("abcd".to_string(),)))
        );
    }

    #[test]
    fn parse_nak_good() {
        let data = ser::to_vec_packed(&("abcd".to_string(), false, 0, 1, 5, 10)).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_nak(message),
            Ok(Some(Message::NAK(
                "abcd".to_string(),
                Some(vec![(0, 1), (5, 10)])
            )))
        );
    }

    #[test]
    fn parse_chunk_good() {
        let bytes = Value::Bytes(vec![0, 0, 1, 1, 2, 2]);
        let data = ser::to_vec_packed(&("abcd".to_string(), 10, bytes)).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_chunk(message),
            Ok(Some(Message::ReceiveChunk(
                "abcd".to_string(),
                10,
                vec![0, 0, 1, 1, 2, 2]
            )))
        );
    }

    #[test]
    fn parse_sync_good() {
        let data = ser::to_vec_packed(&("abcd".to_string(),)).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_sync(message),
            Ok(Some(Message::Sync("abcd".to_string())))
        );
    }

    #[test]
    fn parse_metadata_good() {
        let data = ser::to_vec_packed(&("abcd".to_string(), 100)).unwrap();

        let message = de::from_slice(&data).unwrap();

        assert_eq!(
            parse_sync(message),
            Ok(Some(Message::Metadata("abcd".to_string(), 100)))
        );
    }
}
