use mdns_sd::{ServiceDaemon, ServiceInfo};
use std::sync::Mutex;

static MDNS_STATE: Mutex<Option<MdnsState>> = Mutex::new(None);

struct MdnsState {
    daemon: ServiceDaemon,
    fullname: String,
}

/// Publish an mDNS/Bonjour HTTP service so the share is discoverable via `hostname.local`.
pub fn publish_service(hostname: &str, host_address: &str, port: u16) -> String {
    let mut clean_host = hostname.trim().to_string();
    if clean_host.is_empty() {
        clean_host = "thlauh".to_string();
    }
    // Strip `.local` suffix if user included it
    if clean_host.to_lowercase().ends_with(".local") {
        clean_host = clean_host[..clean_host.len() - 6].to_string();
    }

    let mdns_hostname = format!("{}.local", clean_host);

    // Stop any existing service first
    unpublish_service();

    let daemon = match ServiceDaemon::new() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Failed to create mDNS daemon: {}", e);
            return mdns_hostname;
        }
    };

    let service_type = "_http._tcp.local.";
    let instance_name = &clean_host;

    // Build the service info
    let host_fqdn = format!("{}.", mdns_hostname); // mDNS requires trailing dot
    let service = match ServiceInfo::new(
        service_type,
        instance_name,
        &host_fqdn,
        host_address,
        port,
        None,
    ) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create mDNS ServiceInfo: {}", e);
            return mdns_hostname;
        }
    };

    let fullname = service.get_fullname().to_string();

    if let Err(e) = daemon.register(service) {
        eprintln!("Failed to register mDNS service: {}", e);
    }

    if let Ok(mut state) = MDNS_STATE.lock() {
        *state = Some(MdnsState {
            daemon,
            fullname,
        });
    }

    mdns_hostname
}

/// Unpublish (stop) the mDNS service if one is active.
pub fn unpublish_service() {
    if let Ok(mut state) = MDNS_STATE.lock() {
        if let Some(s) = state.take() {
            let _ = s.daemon.unregister(&s.fullname);
            let _ = s.daemon.shutdown();
        }
    }
}
