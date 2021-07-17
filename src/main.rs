mod discord;
use std::thread;
use std::sync::mpsc;
use std::time::Duration;

fn create_thread() -> thread::JoinHandle<i32> {
    thread::spawn(move || {
        loop {
            println!("side process");
            thread::sleep(Duration::from_millis(2500));
        }
    })
}

fn main() {
    let (tx, rx) = mpsc::sync_channel(0);

    // transmitter thread
    let transmitter = thread::spawn(move || {
        let mut n: i32 = 0;
        loop {
            match tx.send(n) {
                Ok(_) => {},
                Err(_) => println!("unreceived signal!")
            };
            thread::sleep(Duration::from_millis(1000));
            n = n + 1;
        }
    });

    // receiver thread
    let receiver = thread::spawn(move || {
        loop {
            println!("{:?}", rx.recv().unwrap());
            thread::sleep(Duration::from_millis(2000));
        }
    });

    transmitter.join().unwrap();
    receiver.join().unwrap();
}