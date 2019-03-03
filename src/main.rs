#[macro_use] extern crate crossbeam_channel;

use crossbeam_channel::{bounded, Sender, Receiver, tick};

use std::thread;
use std::time::{Duration};

fn main() {
    let mut life1 = Life::new(1, true);
    let mut life2 = Life::new(2, false);
    let mut life3 = Life::new(3, false);

    life1.peer_senders.push(life2.sender.clone());
    life1.peer_senders.push(life3.sender.clone());

    life2.peer_senders.push(life1.sender.clone());
    life2.peer_senders.push(life3.sender.clone());

    life3.peer_senders.push(life1.sender.clone());
    life3.peer_senders.push(life2.sender.clone());

    thread::spawn(move || {
        life1.start();
    });

    thread::spawn(move || {
        life2.start();
    });

    thread::spawn(move || {
        life3.start();
    });

    println!("START");
    std::thread::sleep(Duration::from_secs(10));
    println!("END");
}

struct Life {
    id: u32,
    exist: bool,
    sender: Sender<HeartBeat>,
    receiver: Receiver<HeartBeat>,
    peer_senders: Vec<Sender<HeartBeat>>
}

#[derive(Debug)]
struct HeartBeat {
    id: u32,
    exist: bool,
}

impl Life {
    fn new(id: u32, exist: bool) -> Life {
        let (s, r) = bounded(10);
        Life{
            id,
            exist,
            sender: s,
            receiver: r,
            peer_senders: Vec::<Sender<HeartBeat>>::new(),
        }
    }

    fn start(&self) {
        let ticker = tick(Duration::from_millis(500));
        let timeout = tick(Duration::from_secs(10));
        loop {
            select! {
                recv(self.receiver) -> m => {
                    let heartbeat = m.unwrap();
                    println!("{:?}", heartbeat);
                },
                recv(ticker) -> _=> {
                    println!("ticker from {}", self.id);
                    for s in &self.peer_senders {
                        s.send(HeartBeat{id: self.id, exist: self.exist}).unwrap();
                    }
                },
                recv(timeout) -> _ => {
                    break;
                }
            }
        }
    }
}
