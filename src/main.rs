use pnet::{
    datalink::{self, Channel},
    packet::{
        ethernet::{EtherTypes, EthernetPacket},
        ip::IpNextHeaderProtocols,
        ipv4::Ipv4Packet,
        ipv6::Ipv6Packet,
        tcp::TcpPacket,
        udp::UdpPacket,
        Packet,
    },
};

fn main() -> anyhow::Result<()> {
    let args = std::env::args().collect::<Vec<String>>();
    let interface_name = args.get(1).expect("interface name is required");

    let interfaces = datalink::interfaces();
    let iface = interfaces
        .into_iter()
        .filter(|iface| iface.name == *interface_name)
        .next()
        .expect("interface not found");

    println!("{:#?}", iface);

    let (_tx, mut rx) = datalink::channel(&iface, Default::default()).map(|ch| match ch {
        Channel::Ethernet(tx, rx) => (tx, rx),
        _ => unreachable!(),
    })?;

    loop {
        let frame = rx.next()?;
        let packet = EthernetPacket::new(frame).expect("expected an ethernet packet");
        match packet.get_ethertype() {
            EtherTypes::Ipv4 => ipv4_handler(packet),
            EtherTypes::Ipv6 => ipv6_handler(packet),
            _ => println!("Not an Ipv4 or IPv6 packet"),
        }
    }
}

fn ipv4_handler(packet: EthernetPacket<'_>) {
    if let Some(packet) = Ipv4Packet::new(packet.payload()) {
        match packet.get_next_level_protocol() {
            IpNextHeaderProtocols::Tcp => tcp_handler(&packet),
            IpNextHeaderProtocols::Udp => udp_handler(&packet),
            _ => println!("Not a tcp or a udp packet"),
        }
    }
}

fn ipv6_handler(packet: EthernetPacket<'_>) {
    if let Some(packet) = Ipv6Packet::new(packet.payload()) {
        match packet.get_next_header() {
            IpNextHeaderProtocols::Tcp => tcp_handler(&packet),
            IpNextHeaderProtocols::Udp => udp_handler(&packet),
            _ => println!("Not a tcp or a udp packet"),
        }
    }
}

fn tcp_handler(packet: &impl GetEndPoints) {
    if let Some(tcp) = TcpPacket::new(packet.get_payload()) {
        print_packet_info(packet, &tcp, "TCP");
    }
}

fn udp_handler(packet: &impl GetEndPoints) {
    if let Some(udp) = UdpPacket::new(packet.get_payload()) {
        print_packet_info(packet, &udp, "UDP");
    }
}

fn print_packet_info(
    layer3: &impl GetEndPoints,
    layer4: &impl GetEndPoints,
    proto: &str,
) {
    println!(
        "Captured a {proto} packet from {}:{} to {}:{}",
        layer3.get_source(),
        layer4.get_source(),
        layer3.get_destination(),
        layer4.get_destination(),
    );

    let payload = layer4.get_payload();
    for x in payload.iter() {
        if x.is_ascii() && !x.is_ascii_whitespace() {
            print!("{}", *x as char);
        } else {
            // print!(".");
            print!("{:<02X}", x);
        }
    }
    println!("\n================================\n");
}

pub trait GetEndPoints {
    fn get_source(&self) -> String;
    fn get_destination(&self) -> String;
    fn get_payload(&self) -> &[u8];
}

macro_rules! fuck {
    ($ty: ty) => {
        impl GetEndPoints for $ty {
            fn get_source(&self) -> String {
                self.get_source().to_string()
            }
            fn get_destination(&self) -> String {
                self.get_destination().to_string()
            }
            fn get_payload(&self) -> &[u8] {
                self.payload()
            }
        }
    };
}

fuck!(Ipv4Packet<'_>);
fuck!(Ipv6Packet<'_>);
fuck!(TcpPacket<'_>);
fuck!(UdpPacket<'_>);
