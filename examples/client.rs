use serde::{Deserialize, Serialize};
use wsq::{ClientQueue, ServerQueue};

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    item: u64,
}

fn main() {
    println!("Starting");
    let mut srv = ServerQueue::<Item>::new("127.0.0.1:18090").unwrap();

    println!("Started");
    let cli1 = ClientQueue::<Item>::new("ws://127.0.0.1:18090").unwrap();
    let cli2 = ClientQueue::<Item>::new("ws://127.0.0.1:18090").unwrap();
    let cli3 = ClientQueue::<Item>::new("ws://127.0.0.1:18090").unwrap();

    println!("{:?}", cli1.push(Item { item: 1 }));
    println!("{:?}", cli1.push(Item { item: 2 }));
    println!("{:?}", cli2.push(Item { item: 2 }));
    println!("{:?}", cli1.push(Item { item: 3 }));
    println!("{:?}", cli2.push(Item { item: 3 }));
    println!("{:?}", cli3.push(Item { item: 3 }));
    std::thread::sleep(std::time::Duration::from_millis(10));
    println!("{:?}", srv.pop());
    println!("{:?}", srv.pop());
    println!("{:?}", srv.pop());
    println!("{:?}", srv.pop());
    println!("{:?}", srv.pop());
    println!("{:?}", srv.pop());

    println!("{:?}", cli1.push(Item { item: 1 }));
    println!("{:?}", cli1.push(Item { item: 2 }));
    println!("{:?}", cli2.push(Item { item: 2 }));
    println!("{:?}", cli1.push(Item { item: 3 }));
    println!("{:?}", cli2.push(Item { item: 3 }));
    println!("{:?}", cli3.push(Item { item: 3 }));
    std::thread::sleep(std::time::Duration::from_millis(10));
    for _ in 0..6 {
        let item = srv.pop().unwrap();
        println!("{:?}", srv.push(item));
    }
    std::thread::sleep(std::time::Duration::from_millis(10));
    println!("1: {:?}", cli1.pop());
    println!("1: {:?}", cli1.pop());
    println!("2: {:?}", cli2.pop());
    println!("1: {:?}", cli1.pop());
    println!("2: {:?}", cli2.pop());
    println!("3: {:?}", cli3.pop());
}
