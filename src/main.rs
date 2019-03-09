#[macro_use] extern crate crossbeam_channel;
extern crate rand;

use crossbeam_channel::{unbounded, Sender, Receiver, tick};

use std::thread;
use std::time::Duration;
use std::collections::HashMap;

const WIDTH: usize = 10;
const HEIGHT: usize = 10;

const TIMEOUT: u64 = 10;

fn main() {
    // create life game renderer
    let mut renderer = Renderer::new();

    // create life game board
    let mut table = Vec::<Vec<Life>>::new();

    // create lives
    for i in 0 .. HEIGHT {
        let mut row = Vec::<Life>::new();
        for j in 0 .. WIDTH {
            row.push(Life::new((i*WIDTH+j) as u32, rand::prelude::random(), renderer.sender.clone()));
        }
        table.push(row);
    }

    // lives get peer heart beats
    let positions = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)];
    for i in 0 .. HEIGHT {
        for j in 0 .. WIDTH {
            for position in &positions {
                if let Some(y) = usize_offset(i, HEIGHT, position.0) {
                    if let Some(x) = usize_offset(j, WIDTH, position.1) {
                        let s = table[y][x].sender.clone();
                        table[i][j].peer_senders.push(s);
                    }
                }
            }
        }
    }

    // initialize
    for i in 0 .. HEIGHT {
        for j in 0..WIDTH {
            table[i][j].send();
        }
    }

    // start lives
    while let Some(mut row) = table.pop() {
        while let Some(mut col) = row.pop() {
            thread::spawn(move || {
                col.start();
            });
        }
    }

    std::thread::sleep(Duration::from_millis(100));

    renderer.start();
}

fn usize_offset(curr: usize, max: usize, offset: i32) -> Option<usize> {
    let pos = curr as i32 + offset;
    if pos >= 0 && pos < max as i32 {
        Some(pos as usize)
    } else {
        None
    }
}

struct Life {
    id: u32,
    exist: bool,
    sender: Sender<HeartBeat>,
    receiver: Receiver<HeartBeat>,
    peer_senders: Vec<Sender<HeartBeat>>,
    renderer_sender: Sender<HeartBeat>,
    peer_table: HashMap<u32, bool>,
}

#[derive(Debug)]
struct HeartBeat {
    id: u32,
    exist: bool,
}

impl Life {
    fn new(id: u32, exist: bool, renderer_sender: Sender<HeartBeat>) -> Life {
        let (s, r) = unbounded();
        Life{
            id,
            exist,
            sender: s,
            receiver: r,
            peer_senders: Vec::<Sender<HeartBeat>>::new(),
            renderer_sender,
            peer_table: HashMap::new(),
        }
    }

    fn start(&mut self) {
        let ticker = tick(Duration::from_millis(1000));
        let timeout = tick(Duration::from_secs(TIMEOUT));
        loop {
            select! {
                recv(self.receiver) -> m => {
                    let heartbeat = m.unwrap();
                    self.peer_table.insert(heartbeat.id, heartbeat.exist);
                },
                recv(ticker) -> _=> {
                    self.send();
                    self.update();
                },
                recv(timeout) -> _ => {
                    break;
                }
            }
        }
    }

    fn send(&self) {
        for s in &self.peer_senders {
            match s.send(HeartBeat{id: self.id, exist: self.exist}) {
                Err(e) => eprintln!("{}", e),
                _ => ()
            }
        }
        match self.renderer_sender.send(HeartBeat{id: self.id, exist: self.exist}) {
            Err(e) => eprintln!("{}", e),
            _ => ()
        }
    }

    fn update(&mut self) {
        let mut count = 0;
        for (_k, v) in &self.peer_table {
            if *v {
                count += 1;
            }
        }
        if self.exist {
            if count <= 1 {
                self.exist = false;
            } else if count == 2 || count == 3 {
                self.exist = true;
            } else if count >= 4 {
                self.exist = false;
            }
        } else {
            if count == 3 {
                self.exist = true;
            }
        }
    }
}

struct Renderer {
    sender: Sender<HeartBeat>,
    receiver: Receiver<HeartBeat>,
    table: [bool; (WIDTH*HEIGHT)as usize]
}

impl Renderer {
    fn new() -> Renderer {
        let (sender, receiver) = unbounded();
        Renderer{
            sender,
            receiver,
            table: [false; (WIDTH*HEIGHT)as usize],
        }
    }

    fn start(&mut self) {
        let ticker = tick(Duration::from_millis(1000));
        let timeout = tick(Duration::from_secs(TIMEOUT));
        loop {
            select! {
                recv(self.receiver) -> message => {
                    let heartbeat = message.unwrap();
                    self.table[heartbeat.id as usize] = heartbeat.exist;
                },
                recv(ticker) -> _=> {
                    print!("{}[2J", 27 as char);
                    for i in 0 .. self.table.len() {
                        if i % WIDTH == 0 {
                            print!("\n");
                        }
                        if self.table[i] {
                            print!("■");
                        } else {
                            print!("□");
                        }
                    }
                },
                recv(timeout) -> _ => {
                    break;
                }
            }
        }
    }
}
