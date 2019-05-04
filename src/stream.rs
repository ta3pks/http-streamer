use std::{
    io::Write,
    net::{
        Shutdown,
        TcpListener,
        TcpStream,
    },
    sync::{
        Arc,
        RwLock,
    },
};
pub struct Streamer
{
    clients: RwLock<Vec<TcpStream>>,
}
impl Streamer
{
    pub fn new() -> Arc<Self> //{{{
    {
        Arc::new(Streamer {
            clients: (RwLock::new(Vec::new())),
        })
    }

    //}}}
    pub fn new_client(&self, s: TcpStream) //{{{
    {
        let mut clients = self.clients.write().unwrap();
        clients.push(s);
    }

    //}}}
    pub fn send(&self, data: &str) //{{{
    {
        let mut clients = self.clients.write().unwrap();
        let mut i = 0;
        while i < clients.len()
        {
            let mut sock = &mut clients[i];
            if write!(&mut sock, "data: {}\r\n\n", data).is_err()
            {
                clients.remove(i);
                continue;
            }
            i += 1
        }
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
        let mut clients = self.clients.write().unwrap();
        let mut i = 0;
        while i < clients.len()
        {
            let mut sock = &mut clients[i];
            if write!(&mut sock, "event: {}\r\ndata: {}\r\n\n", event, data).is_err()
            {
                clients.remove(i);
                continue;
            }
            i += 1
        }
    }

    //}}}
    pub fn send_json<T: serde::Serialize>(&self, data: &T) //{{{
    {
        self.send(serde_json::to_string(data).unwrap_or_default().as_str())
    }

    //}}}
    pub fn start(self: Arc<Self>, addr: &str) -> std::io::Result<()> //{{{
    {
        let listener = TcpListener::bind(addr)?;
        std::thread::spawn(move || {
            //{{{
            loop
            {
                let (sock, _addr) = match listener.accept()
                {
                    Ok((sock, _addr)) =>
                    {
                        match sock.set_read_timeout(Some(std::time::Duration::from_millis(200)))
                        {
                            Err(e) =>
                            {
                                eprintln!("error setting timeout{}", e);
                                let _ = sock.shutdown(Shutdown::Both);
                                continue;
                            }
                            Ok(_) => (sock, _addr),
                        }
                    }
                    Err(e) =>
                    {
                        eprintln!("error accepting the client to streaming endpoint {}", e);
                        continue;
                    }
                };
                let mut sock = sock;
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
