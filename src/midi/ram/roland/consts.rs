use super::controller_route::ControllerRoutesTable;
use super::drum_setup::DrumSetup;

pub const DEFAULT_SYSTEM_PARAMS: [u8; 0x80] = {
    let mut data = [0u8; 0x80];
    data[0x04] = 0x7F;
    data[0x05] = 0x40;
    data[0x07] = 0x7F;
    data[0x08] = 0x7F;
    data[0x09] = 0x7F;
    data
};

pub const DEFAULT_EFFECT_PARAMS: [u8; 0x80] = {
    let mut data = [0u8; 0x80];
    data[0x01] = 0x7F;
    data[0x02] = 0x40;
    data[0x03] = 0x0A;
    data[0x04] = 0x3C;
    data[0x30] = 0x04;

    data[0x21] = 90;
    data[0x22] = 50;
    data[0x23] = 40;
    data[0x24] = 30;
    data[0x25] = 40;
    data[0x38] = 2;
    data
};

pub const DEFAULT_PARTS_PARAMS: [[u8; 0x80]; 0x10] = {
    let mut data = [[0u8; 0x80]; 0x10];
    let mut part = 0; // 0 means channel 10, drum channel
    while part < 16 {
        let mut part_params = data[part];
        part_params[0x14] = 1;
        part_params[0x15] = if part == 0 { 1 } else { 0 };
        part_params[0x16] = 0x40;
        part_params[0x17] = 0x40;
        part_params[0x19] = 0x64;
        part_params[0x1A] = 0x40;
        part_params[0x1B] = 0x7F;
        part_params[0x1C] = 0x40;
        part_params[0x1D] = 0x40;
        part_params[0x24] = 0x02;

        let mut i = 0x30;
        while i < 0x35 {
            part_params[i] = 0x40;
            i += 1;
        }

        let mut i = 0x40;
        while i < 0x4C {
            part_params[i] = 0x40;
            i += 1;
        }

        data[part] = part_params;
        part += 1;
    }
    data
};

pub const DEFAULT_CONTROLLER_ROUTES: [ControllerRoutesTable; 0x10] =
    [ControllerRoutesTable::new(); 0x10];

pub const DEFAULT_DRUM_SETUP: [[DrumSetup; 128]; 2] = [[DrumSetup::new(); 128]; 2];
