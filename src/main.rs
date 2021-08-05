use core::time;
use std::{io::{self, Read, Write}, net::{TcpListener, TcpStream, SocketAddr}, str::from_utf8, sync::{Arc, Mutex}, thread, vec};
use clap::{Arg, App};

#[allow(dead_code)]
struct ClientInfo {
    addr: TcpStream,
    username: String
}

fn handle_client(mut stream: TcpStream, clients: Arc<Mutex<Vec<ClientInfo>>>) {

    println!("client connected");

    let ip = stream.peer_addr().unwrap().ip();
    let port = stream.peer_addr().unwrap().port();
    println!("ip: {}, port: {}", ip.to_string(), port.to_string());

    stream.write("enter username:".as_bytes()).unwrap();
    stream.flush().unwrap();

    let mut ufs = "".to_string();

    let mut data = vec![0 as u8; 1024];
    match stream.read(&mut data) {
        Ok(size) => {
            let text = from_utf8(&data[0..size-2]).unwrap();
            ufs = text.clone().to_string();
            let clientinfo = ClientInfo {
                addr: stream.try_clone().expect("clone failed..."),
                username: text.clone().to_string()
            };
            println!("{} connected", text.clone());
            {
                let clientslist = &mut *clients.lock().unwrap();
                clientslist.push(clientinfo);
            }
        },
        Err(_) => {
            
        }
    }

    loop {
        let mut data = vec![0 as u8; 1024];
        match stream.read(&mut data) {
            Ok(size) => {
                let text = from_utf8(&data[0..size-2]).unwrap();
                println!("{}", text.clone());
                let vec = text.clone().split(':').collect::<Vec<&str>>();
                let username = vec[0].clone();
                {
                    let clientslist = clients.lock().unwrap();
                    for c in &*clientslist {
                        if c.username == username.to_string() {
                            let cc = c.addr.try_clone();
                            match cc {
                                Ok(mut v) => {
                                    let mut msg = "".to_string();

                                    msg.push_str(&ufs.clone().to_string());
                                    msg.push_str("::");

                                    msg.push_str(vec[1].clone());
                                    
                                    v.write(msg.as_bytes()).unwrap();
                                    v.flush().unwrap();  
                                },
                                Err(_) => {

                                }
                            }
                        }
                    }
                }
            },
            Err(_) => {
                
            }
        }
    };

}

fn main() -> std::io::Result<()> {

    let matches = App::new("lolo chat")
    .version("0.1")
    .author("Massoud Asadi | massoud.asadi@hotmail.com")
    .about("chat app")
    .arg(Arg::new("type")
        .short('t')
        .long("type")
        .about("client or server")
        .takes_value(true)
        .required(true)
    )
    .arg(Arg::new("ip")
        .short('i')
        .long("ip")
        .about("ip address")
        .takes_value(true)
    )
    .arg(Arg::new("port")
        .short('p')
        .long("port")
        .about("port address")
        .takes_value(true)
    )
    .get_matches();

    if let Some(t) = matches.value_of("type") {

        if t == "server" {
    
            let clients = Arc::new(Mutex::new(Vec::new()));

            let saddr;

            if let Some(p) = matches.value_of("port"){
                saddr = SocketAddr::from(([127,0,0,1], p.parse::<u16>().unwrap()));
            }
            else {
                saddr = SocketAddr::from(([127,0,0,1], 33222));
            }

            let listener = TcpListener::bind(saddr)
            .expect("error in binding tcp socket");

            println!("lolo started at {}", listener.local_addr().unwrap());

            for stream in listener.incoming() {
                match stream {
                    Ok(s) => {

                        let clients = Arc::clone(&clients);

                        thread::spawn(move|| {

                            handle_client(s, clients);

	        			});
                    }
                    Err(e) => panic!("encountered IO error: {}", e),
                }
            }

            drop(listener);

        }
        else {

            let cip;
            let cport;
            let mut  ipport = "".to_string();

            if let Some(ip) = matches.value_of("ip"){
                cip = ip;
                ipport.push_str(cip);
                ipport.push_str(":");
            }

            if let Some(port) = matches.value_of("port"){
                cport = port;
                ipport.push_str(cport);
            }

            println!("{}", ipport);

            match TcpStream::connect(ipport) {
                Ok(stream) => {
                    println!("lolo connected to {}", stream.peer_addr().unwrap());
                    let mut read_clone = stream.try_clone().expect("clone failed...");
                    let mut write_clone = stream.try_clone().expect("clone failed...");
                    thread::spawn(move|| {
                        loop {
                            let mut buffer = String::new();
                            let stdin = io::stdin();
                            stdin.read_line(&mut buffer).unwrap();
                            write_clone.write(buffer.as_bytes()).unwrap(); 
                            write_clone.flush().unwrap();                     
                        };
                    });
                    thread::spawn(move|| {
                        loop {
                            let mut data = vec![0 as u8; 1024];
                            match read_clone.read(&mut data) {
                                Ok(size) => {
                                    let text = from_utf8(&data[0..size]).unwrap();
                                    println!("{}", text);
                                },
                                Err(_) => {
                                    
                                }
                            }
                        };
                    });
                    loop {
                        thread::sleep(time::Duration::from_secs(1));
                    }
                },
                Err(e) => {
                    println!("Failed to connect: {}", e);
                }
            }

        }
    }

    Ok(())
}