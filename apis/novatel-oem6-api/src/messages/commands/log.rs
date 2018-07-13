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

use super::*;

pub struct LogCmd {
    hdr: Header,
    port: u32,
    msg_id: u16,
    msg_type: u8,
    trigger: LogTrigger,
    period: f64,
    offset: f64,
    hold: bool,
}

impl LogCmd {
    pub fn new(
        port: u32,
        msg_id: u16,
        trigger: LogTrigger,
        period: f64,
        offset: f64,
        hold: bool,
    ) -> Self {
        LogCmd {
            hdr: Header::new(MessageID::Log, 32),
            port,
            msg_id,
            msg_type: 0,
            trigger,
            period,
            offset,
            hold,
        }
    }
}

impl Message for LogCmd {
    fn serialize(&self) -> Vec<u8> {
        let mut vec = vec![];

        //Add header
        vec.append(&mut self.hdr.serialize());

        // Add LOG message
        vec.write_u32::<LittleEndian>(self.port).unwrap();
        vec.write_u16::<LittleEndian>(self.msg_id).unwrap();
        vec.push(self.msg_type);
        vec.push(0x00); // Reserved byte
        vec.write_u32::<LittleEndian>(self.trigger as u32).unwrap();
        vec.write_f64::<LittleEndian>(self.period).unwrap();
        vec.write_f64::<LittleEndian>(self.offset).unwrap();
        vec.write_u32::<LittleEndian>(self.hold as u32).unwrap();

        vec
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum LogTrigger {
    OnChanged = 1,
    OnTime = 2,
    Once = 4,
}
