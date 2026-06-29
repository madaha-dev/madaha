use std::fs::read;
use std::io::Error;

use crate::tbl::yxg2006le::decrypt::decrypt;



pub fn load_tbl_file(bin_path: String, waveform_path: String) -> Result<(), Error>{
    let bin = read(bin_path)?;
    let mut waveform = read(waveform_path)?;
    decrypt(&mut waveform);
    
    Ok(())
}