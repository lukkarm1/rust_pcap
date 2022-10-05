use std::error::Error;
use std::fs::{File, Metadata};
use std::io::{BufReader, Read, ErrorKind, Seek};
use std::mem::size_of;

pub struct PcapFile {
    pub global_header: PcapHeader,
    pub packets: Vec<Packet>,
    pub metadata: Metadata,
}

impl std::fmt::Debug for PcapFile {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PcapFile")
            .field("global_header", &self.global_header)
            .field("packets (count): ", &self.packets.len())
            .field("metadata", &self.metadata)
            .finish()
    }
}

#[derive(Default, PartialEq)]
pub struct PcapHeader {
    pub magic_number: u32,
    pub version_major: u16,
    pub version_minor: u16,
    pub thiszone: i32,
    pub sigfigs: u32,
    pub snaplen: u32,
    pub network: u32,
}

impl std::fmt::Debug for PcapHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("PcapHeader")
            .field("magic_number", &format_args!("{0:?} (0x{0:X})", &self.magic_number))
            .field("version_major", &self.version_major)
            .field("version_minor", &self.version_minor)
            .field("thiszone", &self.thiszone)
            .field("sigfigs", &self.sigfigs)
            .field("snaplen", &self.snaplen)
            .field("network", &self.network)
            .finish()
    }
}

#[derive(Default, Debug)]
pub struct Packet {
    header: PacketHeader,
    data: PacketData,
}

#[derive(Default, Debug)]
struct PacketHeader {
    ts_sec: u32,
    ts_usec: u32,
    incl_len: u32,
    orig_len: u32,
}

#[derive(Default, Debug)]
struct PacketData {
    data: Vec<u8>,
}

impl PcapFile {
    pub fn read_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let metadata = file.metadata()?;
        let mut reader = BufReader::new(file);
        let file_header = dbg!(Self::read_file_header(&mut reader)?);
        let packets = Self::read_packets(&mut reader)?;
        Ok(Self {
            global_header: file_header,
            packets,
            metadata,
        })
    }

    fn read_file_header<R: Read>(file: &mut R) -> Result<PcapHeader, Box<dyn Error>> {
        let mut file_header = PcapHeader::default();
        let mut buffer = [0u8; size_of::<PcapHeader>()];

        file.read_exact(&mut buffer)?;
        // The reading application will read either 0xa1b2c3d4 (identical) or 0xd4c3b2a1 (swapped)
        file_header.magic_number = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
        file_header.version_major = u16::from_le_bytes(buffer[4..6].try_into().unwrap());
        file_header.version_minor = u16::from_le_bytes(buffer[6..8].try_into().unwrap());
        file_header.thiszone = i32::from_le_bytes(buffer[8..12].try_into().unwrap());
        file_header.sigfigs = u32::from_le_bytes(buffer[12..16].try_into().unwrap());
        file_header.snaplen = u32::from_le_bytes(buffer[16..20].try_into().unwrap());
        file_header.network = u32::from_le_bytes(buffer[20..=23].try_into().unwrap());

        Ok(file_header)
    }

    fn read_packets<R: Read + Seek>(file: &mut R) -> Result<Vec<Packet>, Box<dyn Error>> {
        let mut packets: Vec<Packet> = vec![];
        // Just example. Maybe some functional tricks could be nicer i.e. packets = collect(...)
        loop {
            let packet = Self::read_packet(file);

            match packet {
                Ok(packet) => packets.push(packet),
                Err(error) => match error.kind() { 
                    ErrorKind::UnexpectedEof => {
                        break;
                    },
                    _ =>{ return Err(Box::new(error)); }
                }
            }
        }
        Ok(packets)
    }

    fn read_packet<R: Read>(file: &mut R) -> Result<Packet, std::io::Error> {
        let header = Self::read_packet_header(file)?;

        let mut data = Vec::<u8>::new();
        data.resize(header.incl_len as usize, 0);
        file.read_exact(&mut data)?;
        Ok(Packet {
            header,
            data: PacketData { data },
        })
    }

    fn read_packet_header<R: Read>(file: &mut R) -> Result<PacketHeader, std::io::Error> {
        let mut packet_header = PacketHeader::default();
        let mut buffer = [0u8; size_of::<PacketHeader>()];

        file.read_exact(&mut buffer)?;
        packet_header.ts_sec = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
        packet_header.ts_usec = u32::from_le_bytes(buffer[4..8].try_into().unwrap());
        packet_header.incl_len = u32::from_le_bytes(buffer[8..12].try_into().unwrap());
        packet_header.orig_len = u32::from_le_bytes(buffer[12..=15].try_into().unwrap());

        Ok(packet_header)
    }
}
