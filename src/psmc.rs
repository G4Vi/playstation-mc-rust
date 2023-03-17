use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;

const BLOCK_SIZE : usize = 0x2000;
const FRAME_SIZE : usize = 0x80;

// TODO: switch to CStr::from_bytes_until_nul when stable instead of rolling our own conversion
// heavily based on:
// https://stackoverflow.com/questions/42066381/how-to-get-a-str-from-a-nul-terminated-byte-slice-if-the-nul-terminator-isnt
fn string_from_u8_nul_utf8(utf8_src: &[u8]) -> Result<String, std::string::FromUtf8Error> {
    let nul_range_end = utf8_src.iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len()); // default to length if no `\0` present
    return String::from_utf8(utf8_src[0..nul_range_end].to_vec());
}

pub struct Save {
    filename: String,
    blocks : Vec<[u8; BLOCK_SIZE]>
}

impl Save {
    fn from_mcs(buffer : &Vec<u8>) -> Save {
        // read in the header (directory frame)
        let header: [u8; FRAME_SIZE] = match buffer[0 .. FRAME_SIZE].try_into() {
            Err(_) => panic!("mcs header is incomplete"),
            Ok(res) => res
        };
        let data_size = ((header[0x04] as u32) | ((header[0x05] as u32) << 8)
        | ((header[0x06] as u32) << 16) | ((header[0x07] as u32) << 24)) as usize;
        let filename_err_message = "mcs filename is not ascii";
        let filename = match string_from_u8_nul_utf8(&header[0xA .. 0x1E]) {
            Err(_) => panic!("{}", filename_err_message),
            Ok(res) => res
        };
        if! filename.is_ascii() {
            panic!("{}", filename_err_message);
        }

        // read in the data
        let chunks = buffer[ FRAME_SIZE .. ].chunks_exact(BLOCK_SIZE);
        if chunks.remainder().len() != 0 {
            panic!("mcs data is not a multiple of {}", BLOCK_SIZE);
        }
        let data : Vec<[u8; BLOCK_SIZE]> = chunks.map(|s| {
            return s.try_into().unwrap();
            //let arr : [u8; BLOCK_SIZE] = s.try_into().unwrap();
            //return arr;
        }).collect();

        if data_size != (data.len()*BLOCK_SIZE) {
            panic!("mcs data_size vs file size mismatch");
        }
        println!("Found save {} with {} bytes", filename, data_size);
        return Save{filename: filename, blocks: data};
    }

    pub fn to_raw(&self) {
        let mut outf = File::create(&self.filename).unwrap();
        for block in &self.blocks {
            outf.write_all(block).unwrap();
        }
        outf.flush().unwrap();
    }
}

pub struct MemoryCard {
    pub saves : Vec<Save>
}

impl MemoryCard {
    // Currently just handles MCS files
    pub fn from_files(files : &[File]) -> MemoryCard {
        let mut saves: Vec<Save> = Vec::new();
        for f in files {
            let mut reader = BufReader::new(f);
            let mut buffer = Vec::new();
            match reader.read_to_end(&mut buffer) {
                Ok(_) => (),
                Err(_) => panic!("read error")
            };
            if (buffer.len() >= FRAME_SIZE) && (((buffer.len() - FRAME_SIZE) % BLOCK_SIZE) == 0) {
                saves.push(Save::from_mcs(&buffer));
            } else {
                panic!("Unsupported file format");
            }
        }
        return MemoryCard{ saves: saves};
    }
}