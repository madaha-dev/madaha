use super::controller_route::ControllerRoutesTable;
use super::drum_setup::DrumSetup;
use super::part::Part;

pub const DEFAULT_PARTS_PARAMS: [Part; 0x10] = {
    let mut data = [Part::new(0); 0x10];
    let mut part = 1; // 0 means channel 10, drum channel
    while part < 16 {
        data[part].assign_mode = 1;
        part += 1;
    }
    data
};

pub const DEFAULT_CONTROLLER_ROUTES: [ControllerRoutesTable; 0x10] =
    [ControllerRoutesTable::new(); 0x10];

pub const DEFAULT_DRUM_SETUP: [[DrumSetup; 128]; 2] = [[DrumSetup::new(); 128]; 2];
