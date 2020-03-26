extern crate pnet;

use std::net::IpAddr;

use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self};

use pnet::packet::arp::ArpPacket;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;

use pnet::packet::tcp::TcpPacket;

use crate::pnet::packet::Packet;

trait GenericIpPacket<'a, T> {
    fn create(packet: &'a [u8]) -> Option<T>;
    fn get_payload(&self) -> &[u8];
    fn get_source_addr(&self) -> String;
    fn get_destination_addr(&self) -> String;
}

impl<'a> GenericIpPacket<'a, Ipv4Packet<'a>> for Ipv4Packet<'a> {
    fn create(packet: &'a [u8]) -> Option<Ipv4Packet<'a>> {
        return Ipv4Packet::new(packet);
    }

    fn get_payload(&self) -> &[u8] {
        return self.payload();
    }

    fn get_source_addr(&self) -> String {
        return IpAddr::V4(self.get_source()).to_string();
    }

    fn get_destination_addr(&self) -> String {
        return IpAddr::V4(self.get_destination()).to_string();
    }
}

impl<'a> GenericIpPacket<'a, Ipv6Packet<'a>> for Ipv6Packet<'a> {
    fn create(packet: &'a [u8]) -> Option<Ipv6Packet<'a>> {
        return Ipv6Packet::new(packet);
    }

    fn get_payload(&self) -> &[u8] {
        return self.payload();
    }
    fn get_source_addr(&self) -> String {
        return IpAddr::V6(self.get_source()).to_string();
    }

    fn get_destination_addr(&self) -> String {
        return IpAddr::V6(self.get_destination()).to_string();
    }
}

impl<'a> GenericIpPacket<'a, ArpPacket<'a>> for ArpPacket<'a> {
    fn create(packet: &'a [u8]) -> Option<ArpPacket<'a>> {
        return ArpPacket::new(packet);
    }

    fn get_payload(&self) -> &[u8] {
        return self.payload();
    }
    fn get_source_addr(&self) -> String {
        return self.get_sender_proto_addr().to_string();
    }

    fn get_destination_addr(&self) -> String {
        return self.get_target_proto_addr().to_string();
    }
}

fn log_ethernet_packet<'a, T: GenericIpPacket<'a, T>>(interface_name: &str, packet: &'a [u8]) {
    match T::create(packet) {
        Some(ip_packet) => match TcpPacket::new(ip_packet.get_payload()) {
            Some(tcp_packet) => println!(
                "[{}]  {} PORT {} to  {} PORT {}",
                interface_name,
                ip_packet.get_source_addr(),
                tcp_packet.get_source(),
                ip_packet.get_destination_addr(),
                tcp_packet.get_destination()
            ),
            None => println!("Unable to parse packet"),
        },
        None => println!("Unable to parse packet"),
    }
}

fn main() {
    let interface_name = "br-f8888a63c67d";
    let interfaces = datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .find(|a| a.name == interface_name)
        .expect("Can not find interface");

    let (_, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled interface type"),
        Err(e) => panic!("Unable to receive packet: {}", e),
    };

    println!("Interface name: {} ", interface.name);
    loop {
        match rx.next() {
            Ok(packet) => {
                let ether_packet = EthernetPacket::new(packet).unwrap();
                match ether_packet.get_ethertype() {
                    EtherTypes::Ipv4 => {
                        log_ethernet_packet::<Ipv4Packet>(interface_name, &ether_packet.payload())
                    }
                    EtherTypes::Ipv6 => {
                        log_ethernet_packet::<Ipv6Packet>(interface_name, &ether_packet.payload())
                    }
                    EtherTypes::Arp => {
                        log_ethernet_packet::<ArpPacket>(interface_name, &ether_packet.payload())
                    }
                    _ => eprintln!("? Unsupported ethernet type"),
                }
            }
            Err(e) => panic!("Unable to receive packet: {}", e),
        }
    }
}
