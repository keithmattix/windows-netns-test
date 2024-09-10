use std::{io::IoSliceMut, net::SocketAddr};

use windows::Win32::NetworkManagement::IpHelper::{SetCurrentThreadCompartmentId, GetCurrentThreadCompartmentId};
use hcn::{api, get_namespace, schema::*};
use tokio::{io, net::{TcpListener, TcpStream}};
use std::io::{Error, ErrorKind};


pub struct Listener(TcpListener);

impl Listener {
    pub fn new(l: TcpListener) -> Self {
        Self(l)
    }
    pub fn local_addr(&self) -> SocketAddr {
        self.0.local_addr().expect("local_addr is available")
    }
    pub fn inner(self) -> TcpListener {
        self.0
    }
    pub async fn accept(&self) -> io::Result<(TcpStream, SocketAddr)> {
        let (stream, remote) = self.0.accept().await?;
        stream.set_nodelay(true)?;
        Ok((stream, remote))
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // TOOD: Remove the `.0` when the return type is fixed; WIN32_ERROR maps a u32
    // value so this should work since the memory layout is correct.
    // We assume this must not fail.
    let compartment_id : u32 = unsafe { GetCurrentThreadCompartmentId().0 };
    println!("Current compartment id: {}", compartment_id);

    // Query list of namespaces
    let query = HostComputeQuery::default();
    let query = serde_json::to_string(&query).unwrap();
    let namespaces = api::enumerate_namespaces(&query).unwrap();
    let namespaces : Vec<String> = serde_json::from_str(&namespaces).unwrap();
    let ns = get_namespace(namespaces.first().unwrap().as_str()).unwrap();
    println!("First Namespace details: {:?}", ns);
    // Namespaced ID == Compartment ID so we can use that to change our compartment
    unsafe {
        let error = SetCurrentThreadCompartmentId(ns.namespace_id.unwrap());
        if error.0 != 0 {
            panic!("Error setting compartment id: {}", error.0);
        }
        println!("Printing from inside Compartment ID {}", GetCurrentThreadCompartmentId().0);
        // Open a socket
        let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
        let std_sock = std::net::TcpListener::bind(addr).unwrap();
        println!("Socket opened in compartment {}", GetCurrentThreadCompartmentId().0);
        // Change back to original compartment
        let error = SetCurrentThreadCompartmentId(compartment_id);
        if error.0 != 0 {
            panic!("Error setting compartment id: {}", error.0);
        }
        println!("Back in original compartment ID {}", GetCurrentThreadCompartmentId().0);
        std_sock.set_nonblocking(true).unwrap();
        let l = TcpListener::from_std(std_sock).map(Listener::new).unwrap();
        let socket = l.accept().await;
        match socket {
            Ok((stream, remote)) => {
                println!("Remote addr: {:?}", remote);
                let mut buffer: Vec<u8> = vec![0u8; 1024];
                let mut iov = [IoSliceMut::new(&mut buffer)];

                loop {
                    println!("Waiting for socket to be readable");
                    stream.readable().await.unwrap();
                    let res = stream.try_read_vectored(&mut iov);
                    match res {
                        Ok(l) => {
                            println!("Bytes read: {:?}", l);
                            println!("Data received: {:?}", &buffer[..l].to_ascii_lowercase());
                            break;
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            println!("Received WouldBlock error, retrying");
                            continue;
                        }
                        Err(e) => {
                            println!("error received reading from socket: {:?}", e);
                            return
                        }
                    };

                };

            }
            Err(e) => {
                if is_runtime_shutdown(&e) {
                    return;
                }
                panic!("Failed TCP handshake {}", e);
            }
        }
    }


}

fn is_runtime_shutdown(e: &Error) -> bool {
    if e.kind() == ErrorKind::Other
        && e.to_string() == "A Tokio 1.x context was found, but it is being shutdown."
    {
        return true;
    }
    false
}


