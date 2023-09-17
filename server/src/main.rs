use std::io::{ErrorKind,Read,Write};//basic input output
use std::net::TcpListener;//to create and listen to a server
use std::sync::mpsc;//allows to spawn a channel
use std::thread;//use multiple threads



fn sleep(){
    thread::sleep(::std::time::Duration::from_millis(1000));
}
const LOCAL : &str="localhost:4000";
const MSG_SIZE: usize=32;
//msg with greater than 32 bytes storage space required is cut off

fn main() {
    let server=TcpListener::bind(LOCAL).expect("Listener failed to bind");

    //Set server in Non-blocking state to force it to hear for changes all the time
    server.set_nonblocking(true).expect("failed to initialize non-blocking");
    
    //create and store clients coming in
    let mut clients=vec![];
    //The channel function will return a (Sender, Receiver) tuple where all sends will be asynchronous.
    let (tx,rx)=mpsc::channel::<String>();//telling channel that we will send many string type messages
    loop{
        if let Ok((mut socket,addr))=server.accept(){
            println!("Client {0}  connected",addr);
            let tx= tx.clone();
            clients.push(socket.try_clone().expect("failed to clone the client"));

            thread::spawn(move || loop{
                let mut buff=vec![0;MSG_SIZE];
                        //Hear socket entries from sender an match it with a Result.      
                match socket.read_exact( &mut buff){
                    Ok(_)=>{
                        let  msg=buff.into_iter().take_while(|&x| x!= 0).collect::<Vec<_>>();

                        println!("\nMSG as Bytes: {:?}", msg.clone());
                        let  msg = String::from_utf8(msg).expect("Invalid utf8 message");
                        println!("{} : {:?}",addr,msg);//with that debug flag
                        tx.send(msg).expect("failed to send a msg to rx")
                    },
                    Err(ref err)  if err.kind()==ErrorKind::WouldBlock=>(),
                    Err(_)=>{
                        println!("closing the connection with {}",addr);
                        break;
                    },
                }
                sleep();//server would be still running if one client connection is closed,so we make it sleep
            });
        }

        //what happens when we receive a msg
        if  let Ok(msg) = rx.try_recv() {
            clients=clients.into_iter().filter_map(|mut client|{
                let mut buff=msg.clone().into_bytes();//just msg stored in 32 bytes
                buff.resize(MSG_SIZE,0);
                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>()
        }
        sleep();
    
    }
}
