use raikou_window::{RuntimeLifecycle, WindowConfig, WindowRuntime};

struct App;

impl RuntimeLifecycle for App {}

fn main() {
    let config = WindowConfig::default()
        .title("Raikou")
        .initial_size(800.0, 600.0);

    let runtime = WindowRuntime::new(config);
    runtime.run(App).unwrap();
}
