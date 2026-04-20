use bael::{
    app::app::AppRunner, render::wgpu::state::{self, new}, ui::{container::test::Test, widget::Widget}
};

fn main() {
    pretty_env_logger::init();

    let runner = pollster::block_on(AppRunner::new());

    let test = Test::new(bael::render::wgpu::state::State::global().lock().unwrap().first_window_id().unwrap());

    // run the event loop and execute a startup callback on the event-loop thread
    runner.run_with_startup(move || {
        // called on the event-loop thread when it starts; safe to submit to renderer here
        test.show();
    });
}
