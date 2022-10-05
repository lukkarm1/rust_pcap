
fn main() {
    let pcap_file = rust_pcap::PcapFile::read_file("test.pcap").unwrap();
    dbg!(pcap_file);
}
