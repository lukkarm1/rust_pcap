#[test]
fn test_packet_count() {
    let pcap_file = rust_pcap::PcapFile::read_file("tests/test.pcap").unwrap();
    assert_eq!(141, pcap_file.packets.len());
}

#[test]
fn test_header() {
    let pcap_file = rust_pcap::PcapFile::read_file("tests/test.pcap").unwrap();
    let test_header = rust_pcap::PcapHeader {
        magic_number: 0xA1B2C3D4,
        version_major: 2,
        version_minor: 4,
        thiszone: 0,
        sigfigs: 0,
        snaplen: 65535,
        network: 1,
    };
    assert_eq!(test_header, pcap_file.global_header);
}
