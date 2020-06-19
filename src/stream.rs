use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener, TcpStream},
    sync::{Arc, RwLock},
};
pub struct Streamer {
    clients: RwLock<Vec<TcpStream>>,
}
impl Streamer {
    pub fn new() -> Arc<Self> //{{{
    {
        Arc::new(Streamer {
            clients: (RwLock::new(Vec::new())),
        })
    }

    //}}}
    pub fn connected_clients(&self) -> usize {
        self.clients
            .read()
            .map_err(|_| 0)
            .map(|e| e.len())
            .unwrap_or_default()
    }

    pub fn new_client(&self, s: TcpStream) //{{{
    {
        let _ = self.clients.write().and_then(|mut c| Ok(c.push(s)));
    }

    //}}}
    pub fn send(&self, data: &str) //{{{
    {
        let _ = self.clients.write().and_then(|mut clients| {
            let mut i = 0;
            while i < clients.len() {
                let mut sock = &mut clients[i];
                if write!(&mut sock, "data: {}\r\n\n", data).is_err() {
                    clients.remove(i);
                    continue;
                }
                i += 1
            }
            Ok(())
        });
    }

    //}}}
    pub fn send_json_with_event<T: serde::Serialize>(&self, event: &str, data: &T) //{{{
    {
        self.send_with_event(
            event,
            serde_json::to_string(data).unwrap_or_default().as_str(),
        )
    }

    //}}}
    pub fn send_with_event(&self, event: &str, data: &str) //{{{
    {
        let _ = self.clients.write().and_then(|mut clients| {
            let mut i = 0;
            while i < clients.len() {
                let mut sock = &mut clients[i];
                if write!(&mut sock, "event: {}\r\ndata: {}\r\n\n", event, data).is_err() {
                    clients.remove(i);
                    continue;
                }
                i += 1
            }
            Ok(())
        });
    }

    //}}}
    pub fn send_json<T: serde::Serialize>(&self, data: &T) //{{{
    {
        self.send(serde_json::to_string(data).unwrap_or_default().as_str())
    }

    //}}}
    pub fn start<F: FnOnce(String) -> bool + Send + 'static + Clone>(
        self: Arc<Self>,
        addr: &str,
        control_fn: F,
    ) -> std::io::Result<()> //{{{
    {
        let re: regex::Regex = regex::Regex::new(r"GET /[^ ]+").unwrap();
        let listener = TcpListener::bind(addr)?;
        std::thread::spawn(move || {
            //{{{
            loop {
                let (sock, _addr) = match listener.accept() {
                    Ok((sock, _addr)) => {
                        match sock.set_read_timeout(Some(std::time::Duration::from_millis(200))) {
                            Err(e) => {
                                eprintln!("error setting timeout{}", e);
                                let _ = sock.shutdown(Shutdown::Both);
                                continue;
                            }
                            Ok(_) => (sock, _addr),
                        }
                    }
                    Err(e) => {
                        eprintln!("error accepting the client to streaming endpoint {}", e);
                        continue;
                    }
                };
                if sock
                    .set_write_timeout(Some(std::time::Duration::from_millis(50)))
                    .is_err()
                {
                    eprintln!("error setting up socket");
                    continue;
                }
                if sock
                    .set_read_timeout(Some(std::time::Duration::from_millis(50)))
                    .is_err()
                {
                    eprintln!("error setting up socket");
                    continue;
                }
                let mut buf = [0u8; 50];
                let mut sock = sock;
                if sock.read(&mut buf).is_err() {
                    eprintln!("error setting up socket");
                    continue;
                }
                let req = String::from_utf8_lossy(&buf);
                let tkn = re.find(&req);
                if tkn.is_none() {
                    continue;
                }
                let tkn: Vec<&str> = tkn.unwrap().as_str().split("/").collect();
                if tkn.len() != 2 {
                    continue;
                }
                let tkn = tkn[1];
                if !control_fn.clone()(tkn.to_owned()) {
                    let _ = sock.write(b"invalid token\r\n");
                    continue;
                }
                let _ = sock.write(b"HTTP/1.1 200 OK\r\n");
                let _ = sock.write(b"Connection: keep-alive\r\n");
                let _ = sock.write(b"Content-Type: text/event-stream\r\n");
                let _ = sock.write(b"x-content-type-options: nosniff\r\n");
                if sock
                    .write(b"Access-Control-Allow-Origin: *\r\n\r\n")
                    .is_ok()
                {
                    self.new_client(sock);
                };
            }
        }); //}}}
        Ok(())
    } //}}}
}
