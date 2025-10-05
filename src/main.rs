use winit::event_loop::EventLoop;

fn main() {
    env_logger::init();
    glutin_hello_world::main(EventLoop::new().unwrap()).unwrap();
}
