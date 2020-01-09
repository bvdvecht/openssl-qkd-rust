use std::thread;
use std::sync::mpsc;
use mpsc::Sender;

use std::net::Ipv4Addr;
use cqc::hdr::CommHdr;
use qkd_rs::Cqc;

use crate::session;

pub struct Key {
    pub value: Vec<u8>
}

unsafe impl Send for Key {}

impl Key {
    pub fn from(length: usize) -> Key {
        Key {
            value: vec![0; length]
        }
    }

    pub fn write_to_buffer(&self, buffer: *mut u8) {
        println!("copying {} bytes of key into buffer", self.value.len());
        let str_slice = unsafe {
            std::slice::from_raw_parts_mut(buffer, self.value.len())
        };
        str_slice[..self.value.len()].copy_from_slice(&self.value);
    }

    pub fn to_string(&self) -> &[u8] {
        &self.value
    }
}

// CQC header of server, used by client
fn server_hdr() -> CommHdr {
    CommHdr {
        remote_app_id: 10,
        remote_port: 8004,
        remote_node: u32::from(Ipv4Addr::new(127, 0, 0, 1))
    }
}

fn server_thread(tx: Sender<Key>) {
    println!("starting server thread");
    let key_length = session::get_key_length();
    let mut key = Key::from(key_length);

    let cqc = Cqc::new(10, "localhost", 8004);

    // generate key by measuring received EPR halves
    for i in 0..key_length {
        let id = cqc.recv_epr(false);
        let outcome = cqc.measure_qubit(id, false);
        print!("{:x?}", outcome as u8);
        key.value[i] = outcome as u8;
    }
    println!("");
    println!("generated key: {:x?}", key.to_string());

    // send generated key to main thread
    tx.send(key).expect("could not send key through channel");
}

fn client_thread(tx: Sender<Key>) {
    println!("starting client thread");
    let key_length = session::get_key_length();
    let mut key = Key::from(key_length);

    let cqc = Cqc::new(10, "localhost", 8001);

    // generate key by measuring EPR halves
    for i in 0..key_length {
        let id = cqc.create_epr(server_hdr(), false);
        let outcome = cqc.measure_qubit(id, false);
        print!("{:x?}", outcome as u8);
        key.value[i] = outcome as u8;
    }
    println!("");
    println!("generated key: {:x?}", key.to_string());

    // send generated key to main thread
    tx.send(key).unwrap();
}

pub fn spawn_qkd_generator(tx: Sender<Key>) {
    if session::get_is_server() {
        thread::spawn(move || {
            server_thread(tx);
        });
    } else {
        thread::spawn(move || {
            client_thread(tx);
        });
    }
}