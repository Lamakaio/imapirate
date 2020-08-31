use std::{io::{Write, Read, BufReader}, fs::File};
use super::sea_main::{Tile};


pub fn generate_tmx (map : Vec<Vec<Tile>>, filename : &str) {
    let mut output = File::create(filename).unwrap();
    let header_file = File::open("assets/map_gen/header").unwrap();
    let mut buf_reader = BufReader::new(header_file);
    let mut header_str = String::new();
    buf_reader.read_to_string(&mut header_str).unwrap();
    let layer_id = 1;
    write!(output, "{}", header_str).unwrap();
    write_layer(&mut output, |output| {format_map(map, output)}, layer_id);
    write!(output, "</map>").unwrap();
}

fn write_layer<F : FnOnce(&mut File) -> ()>(
    output : &mut File, 
    write_csv : F,
    layer_id : u32,
) -> u32 {
    write!(output,
         "<layer id=\"{}\" name=\"{}\" width=\"128\" height=\"128\">\n<data encoding=\"csv\">\n", 
         layer_id, 
         layer_id).unwrap();
    write_csv(output);
    write!(output, 
        "</data>\n</layer>\n").unwrap();
    layer_id + 1
}
fn format_map(
    map : Vec<Vec<Tile>>,
    output : &mut File, 
) {
    for c in map {
        for l in c {
            write!(output, "{},", l.sprite_id).unwrap();
        }
        std::io::stdout().flush().unwrap();
        write!(output, "\n").unwrap();
    }
}