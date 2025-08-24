use anyhow::{Result, Context};
use std::mem;
use std::net::{IpAddr, Ipv4Addr};
use std::ptr;
use winapi::shared::ifdef::{IfOperStatusUp};
use winapi::shared::ipifcons::IF_TYPE_SOFTWARE_LOOPBACK;
use winapi::shared::netioapi::{GetIfEntry2, MIB_IF_ROW2};
use winapi::shared::ws2def::{AF_INET, AF_INET6, SOCKADDR_IN};
use winapi::shared::ntdef::ULONG;
use winapi::um::iphlpapi::GetAdaptersAddresses;
use winapi::um::iptypes::{IP_ADAPTER_ADDRESSES, GAA_FLAG_INCLUDE_PREFIX};
use crate::network::interface::NetworkInterface;
use crate::network::stats::InterfaceStats;

const ERROR_BUFFER_OVERFLOW: u32 = 111;
const ERROR_SUCCESS: u32 = 0;

pub fn get_network_interfaces() -> Result<Vec<NetworkInterface>> {
    unsafe {
        let family = AF_INET; // AF_UNSPEC for both IPv4 and IPv6
        let flags = GAA_FLAG_INCLUDE_PREFIX;
        let mut buffer_size: ULONG = 15000; // Initial buffer size
        let mut buffer: Vec<u8> = vec![0; buffer_size as usize];
        
        // First call to get the required buffer size
        let mut result = GetAdaptersAddresses(
            family as u32,
            flags,
            ptr::null_mut(),
            buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES,
            &mut buffer_size,
        );

        // Resize buffer if needed
        if result == ERROR_BUFFER_OVERFLOW {
            buffer.resize(buffer_size as usize, 0);
            result = GetAdaptersAddresses(
                family as u32,
                flags,
                ptr::null_mut(),
                buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES,
                &mut buffer_size,
            );
        }

        if result != ERROR_SUCCESS {
            return Err(anyhow::anyhow!("Failed to get network adapters: error code {}", result))
                .context("Windows API GetAdaptersAddresses failed");
        }

        let mut interfaces = Vec::new();
        let mut current_adapter = buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES;

        while !current_adapter.is_null() {
            let adapter = &*current_adapter;
            
            // Convert adapter name and description
            let name = wide_string_to_string(adapter.FriendlyName);
            let description = wide_string_to_string(adapter.Description);
            
            let mut interface = NetworkInterface::new(
                adapter.u.s().IfIndex,
                name,
                description,
            );

            // Get MAC address
            if adapter.PhysicalAddressLength > 0 {
                let mac_bytes = &adapter.PhysicalAddress[..adapter.PhysicalAddressLength as usize];
                interface.mac_address = NetworkInterface::format_mac_address(mac_bytes);
            }

            // Check if interface is up
            interface.is_up = adapter.OperStatus == IfOperStatusUp;
            
            // Check if it's a loopback interface
            interface.is_loopback = adapter.IfType == IF_TYPE_SOFTWARE_LOOPBACK;

            // Get IP addresses
            let mut unicast_addr = adapter.FirstUnicastAddress;
            while !unicast_addr.is_null() {
                let unicast = &*unicast_addr;
                if !unicast.Address.lpSockaddr.is_null() {
                    let sockaddr = &*unicast.Address.lpSockaddr;
                    
                    match sockaddr.sa_family as i32 {
                        AF_INET => {
                            let addr_in = sockaddr as *const _ as *const SOCKADDR_IN;
                            let addr = (*addr_in).sin_addr.S_un;
                            let octets = *addr.S_addr();
                            let ip = IpAddr::V4(Ipv4Addr::from(u32::from_be(octets)));
                            interface.ip_addresses.push(ip);
                        }
                        AF_INET6 => {
                            // IPv6 handling if needed
                        }
                        _ => {}
                    }
                }
                unicast_addr = unicast.Next;
            }

            // Get interface speed using GetIfEntry2
            let mut if_row: MIB_IF_ROW2 = mem::zeroed();
            if_row.InterfaceIndex = adapter.u.s().IfIndex;
            if GetIfEntry2(&mut if_row) == ERROR_SUCCESS {
                interface.speed = if_row.TransmitLinkSpeed;
            }

            interfaces.push(interface);
            current_adapter = adapter.Next;
        }

        Ok(interfaces)
    }
}

fn wide_string_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }

    unsafe {
        let mut len = 0;
        // Limit search to prevent potential infinite loops
        const MAX_STRING_LENGTH: isize = 1024;
        while len < MAX_STRING_LENGTH && *ptr.offset(len) != 0 {
            len += 1;
        }

        if len == 0 {
            return String::new();
        }

        let slice = std::slice::from_raw_parts(ptr, len as usize);
        String::from_utf16_lossy(slice)
    }
}

pub fn get_interface_statistics(interface_index: u32) -> Result<InterfaceStats> {
    unsafe {
        let mut if_row: MIB_IF_ROW2 = mem::zeroed();
        if_row.InterfaceIndex = interface_index;
        
        let result = GetIfEntry2(&mut if_row);
        if result != ERROR_SUCCESS {
            return Err(anyhow::anyhow!("Failed to get interface statistics for index {}: error code {}", interface_index, result))
                .context("Windows API GetIfEntry2 failed - interface may not exist or be accessible");
        }

        let mut stats = InterfaceStats::new(interface_index);
        stats.bytes_received = if_row.InOctets;
        stats.bytes_sent = if_row.OutOctets;
        stats.packets_received = if_row.InUcastPkts + if_row.InNUcastPkts;
        stats.packets_sent = if_row.OutUcastPkts + if_row.OutNUcastPkts;
        stats.errors_in = if_row.InErrors;
        stats.errors_out = if_row.OutErrors;
        
        Ok(stats)
    }
}