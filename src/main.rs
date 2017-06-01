extern crate i3ipc;
use i3ipc::I3EventListener;
use i3ipc::Subscription;
use i3ipc::event::Event;
fn main() {
  let mut listener = I3EventListener::connect().unwrap();

  // subscribe to a couple events.
  let subs = [Subscription::Mode, Subscription::Binding];
  listener.subscribe(&subs).unwrap();

  // handle them
  for event in listener.listen() {
      match event.unwrap() {
          Event::ModeEvent(e) => println!("new mode: {}", e.change),
          Event::BindingEvent(e) => println!("user input triggered command: {}", e.binding.command),
          _ => unreachable!()
      }
  }
}
