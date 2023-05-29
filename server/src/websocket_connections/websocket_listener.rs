use crate::{lobby::Lobby, websocket_connections::connection::{Connection, ConnectionEventListener, self}, log};
use tokio_native_tls::{TlsAcceptor, native_tls::{self, Identity}};
use tokio_tungstenite::tungstenite::{client, Message};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex}, collections::HashMap,
};

use futures_util::{future::{self}, StreamExt, TryStreamExt, SinkExt};

use tokio::sync::mpsc;
use tokio::net::{TcpListener, TcpStream};

pub async fn create_ws_server(
    address: &str, 
    clients: Arc<Mutex<HashMap<SocketAddr, Connection>>>,
    mut event_listener: Box<impl ConnectionEventListener + Send + 'static>,
) {
    let event_listener = Arc::new(Mutex::new(*event_listener));

    // Create the event loop and TCP listener we'll accept connections on.
    let listener = TcpListener::bind(&address).await
        .unwrap_or_else(|_| panic!("Invalid address: {address}"));  //panic if address is invalid
    
    
    // let identity = Identity::from_pkcs12(include_bytes!("../cert.pfx"), "password").unwrap();
    // let acceptor = native_tls::TlsAcceptor::new(identity)

    print!("\x1B[2J\x1B[1;1H"); // Clear terminal
    println!("{}", log::notice("Mafia Server started!\n"));
    println!("Listening on: {}\n", log::important(address));
    println!("Log output:");

    // Handle each incoming connection in a separate task
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr, clients.clone(), event_listener.clone()));
    }

    //this thread will never close i guess
}

pub async fn handle_connection(
    raw_stream: TcpStream, 
    addr: SocketAddr, 
    clients_mutex: Arc<Mutex<HashMap<SocketAddr, Connection>>>, 
    mut listener: Arc<Mutex<impl ConnectionEventListener>>
) {
    //println!("Incoming TCP connection from: {}", addr);
    let ws_stream = tokio_tungstenite::accept_async(raw_stream).await.expect(
        "Error during the websocket handshake occurred"
    ); //if handshake doesnt work panic
    //println!("WebSocket connection established: {}\n", addr);
    
    //sending to client mpsc
    let (transmitter_to_client, mut reciever_to_client) = mpsc::unbounded_channel();

    //sending to clinet tcp
    let (mut to_client, from_client) = ws_stream.split();


    //create connection struct and give it ways to communicate with client
    {
        let mut clients = clients_mutex.lock().unwrap();
        clients.insert(addr, Connection::new(transmitter_to_client, addr)); 
    }
    
    // route between unbounded senders and websockets
    let send_to_client = tokio::spawn(async move {
        while let Some(m) = reciever_to_client.recv().await {
            match to_client.send(Message::text(m.to_json_string())).await{
                Ok(_) => {},
                Err(_) => {break;},
            }
        }
        to_client.close();
    });

    let recieve_from_client = from_client.try_for_each(|message|{
        let clients = clients_mutex.lock().unwrap();
        let connection = clients.get(&addr).unwrap();

        listener.lock().unwrap().on_message(&clients, connection, &message);
            
        future::ok(())
    });

    {
        let mut clients = clients_mutex.lock().unwrap();
        let connection = clients.get(&addr).unwrap();

        listener.lock().unwrap().on_connect(&clients, connection);
    }
    
    futures_util::pin_mut!(send_to_client, recieve_from_client);//pinmut needed for select
    future::select(send_to_client, recieve_from_client).await;

    // When either are complete then that means it's disconnected
    {
        let mut clients = clients_mutex.lock().unwrap();
        let connection = clients.get(&addr).unwrap();
        
        listener.lock().unwrap().on_disconnect(&clients, connection);
        clients.remove(&addr);
    }
    // println!("{} disconnected", &addr);
}
