#[tokio::main]
async fn main() {
    let mut stream = futures::stream::iter(vec![Ok::<_, ()>(1), Ok(2), Ok(3)]);
    use futures::StreamExt;
    loop {
        tokio::select! {
            Some(Ok(2)) = stream.next() => {
                println!("Got 2");
            }
            else => {
                println!("Else branch");
                break;
            }
        }
    }
}
