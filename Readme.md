# Readme

A thread-safe EventSource library for the fellow Rustaceans. 

# Examples

```rust
	
	let s = Streamer::new();
	let s1=s.clone();
	std::thread::spawn(||s1.start("localhost:1234"));
	
	loop
	{
		std::thread::sleep_ms(1000);
		s.send_with_event("myevent","mydata");
	}

```

# Todo

- [x] send_with_event
- [x] send_json
- [ ] new_with_client_timeout
- [ ] new_with_auth
- [ ] resume
