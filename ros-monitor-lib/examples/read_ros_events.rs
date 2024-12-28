use futures::StreamExt;

#[tokio::main]
async fn main() {
    let process_dir = std::env::current_exe().unwrap();
    let process_dir = process_dir.parent().unwrap().parent().unwrap();
    let command = process_dir.join("intrepid-ros-monitor");
    let monitor = ros_monitor_lib::RosMonitor::new(command);
    let stream = monitor.subscribe().unwrap();
    let mut stream = std::pin::pin!(stream);

    while let Some(event) = stream.next().await {
        let _ = dbg!(event);
    }
}
