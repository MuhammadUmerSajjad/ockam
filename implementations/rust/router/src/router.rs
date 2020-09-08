#![allow(unused)]
pub mod router {
    use std::thread;
    use std::collections::HashMap;
    use ockam_message::message::*;
    use std::ops::Add;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::net::UdpSocket;
    use std::io::{Read, Write, Error, ErrorKind};


    pub struct UdpAddressHandler {
        pub socket: UdpSocket,
    }

    impl HandleMessage for UdpAddressHandler {
        fn handle_message(&self, mut m: Box<Message>) -> Result<(), std::io::Error> {
            println!("In UdpAddressHandler!");
            Ok(())
        }
    }

    pub trait HandleMessage {
        fn handle_message(&self, m: Box<Message>) -> Result<(), std::io::Error>;
    }

    pub struct Router<'a> {
        registry: HashMap<u64, &'a dyn HandleMessage>,
    }

    impl<'a> Router<'a> {
        pub fn new() -> Router<'a> {
            Router{ registry: HashMap::new() }
        }

        pub fn register_handler(&mut self, handler: &'a (dyn HandleMessage + 'a), address: &Address)
            -> Result<(), String> {
            let k = address.get_key()?;
            self.registry.insert(k, handler);
            Ok(())
        }

        pub fn route(&mut self, mut m: Box<Message>) -> Result<(), String> {
            // Pop the first address in the list
            // If there are no addresses, route to the controller
            // Controller key is always 0
            let mut key: u64 = 0;
            let address: Address;
            if !m.onward_route.addresses.is_empty() {
                address = m.onward_route.addresses.remove(0);
                match address.get_key() {
                    Ok(k) => { key = k; },
                    Err(s) => { return Err(s) },
                }
            } else {
                address = Address::LocalAddress(LocalAddress{length: 0, address: 0});
            }
            if !self.registry.contains_key(&key) {
                // If no match is found, get key for default handler for address type
                match address {
                    //!!ToDo implement default handler address get_key
                    Address::LocalAddress(a) => { key = 4; },
                    Address::UdpAddress(u, p) => { key = 2; },
                    _ => { return Err("Not Implemented".to_string()) }
                }
            }
            match self.registry.get_mut(&key) {
            Some(mut r) => {
                match r.handle_message(m) {
                    Ok(()) => { Ok(())}
                    Err(e) => { Err(e.to_string())}
                }
            }
            None => {
                Err("key not found".to_string())
                }
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use ockam_message::message::*;
    use std::ops::Add;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
    use std::net::UdpSocket;
    use std::io::{Read, Write, Error, ErrorKind};
    use crate::router::*;

    #[test]
    fn test_handler() {
        let mut onward_addresses: Vec<Address> = vec![];
        onward_addresses.push(Address::UdpAddress(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0x8080));
        onward_addresses.push(Address::UdpAddress(IpAddr::V4(Ipv4Addr::new(10, 0, 1, 10)), 0x7070));
        onward_addresses.push(Address::LocalAddress(LocalAddress { length: 4, address: 0x00010203 }));
        let mut return_addresses: Vec<Address> = vec![];
        return_addresses.push(Address::UdpAddress(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 2)), 0x8080));
        return_addresses.push(Address::UdpAddress(IpAddr::V4(Ipv4Addr::new(10, 0, 1, 11)), 0x7070));
        return_addresses.push(Address::LocalAddress(LocalAddress { length: 4, address: 0x00010203 }));
        let onward_route = Route { addresses: onward_addresses };
        let return_route = Route { addresses: return_addresses };
        let mut message_body = vec![0];
        let mut msg = Box::new(Message {
            version: WireProtocolVersion { v: 1 },
            onward_route,
            return_route,
            message_body
        });
        let mut router: Router = Router::new();
        let udp_socket = UdpSocket::bind("127.0.0.1:4050").expect("couldn't bind to address");
        let udp_handler = UdpAddressHandler{socket: udp_socket};
        match router.register_handler(&udp_handler, &msg.onward_route.addresses[0]) {
            Ok(()) => {
                println!("udp handler registered");
            }
            Err(s) => {println!("{}", s); return;}
        }
        match router.route(msg) {
            Ok(()) => { println!("success!") }
            Err(s) => { println!("{}", s) }
        }
    }
}