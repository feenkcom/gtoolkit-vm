use std::env;
use std::time::Duration;
use vm_runtime::vm_bindings::InterpreterConfiguration;
use vm_runtime::{android_activity, Constellation, VirtualMachineConfiguration};

#[no_mangle]
pub extern "C" fn android_main(app: android_activity::AndroidApp) {
    env::set_var("RUST_LOG", "error");

    std::thread::sleep(Duration::from_secs(1));
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Error),
    );

    let current_exe = env::current_exe().expect("Get current exe");
    let current_dir = env::current_dir().expect("Get current dir");
    let internal_path = app.internal_data_path().expect("Get internal data path");
    let external_path = app.external_data_path().expect("Get external data path");

    println!("current_exe: {}", current_exe.display());
    println!("current_dir: {}", current_dir.display());
    println!("internal_path: {}", internal_path.display());
    println!("external_path: {}", external_path.display());

    let image_path = external_path
        .join("glamoroustoolkit")
        .join("GlamorousToolkit.image");

    {
        let new_current_dir = image_path.parent().expect("Get parent directory");
        if !new_current_dir.exists() {
            panic!(".image directory does not exist");
        }
        env::set_current_dir(new_current_dir).unwrap_or_else(|error| {
            panic!(
                "Set current dir to {}: {}",
                new_current_dir.display(),
                error
            )
        });
    }

    let mut extra_args = vec![];
    extra_args.push("--event-fetcher=winit".to_string());

    let mut interpreter_configuration = InterpreterConfiguration::new(image_path);
    interpreter_configuration.set_interactive_session(true);
    interpreter_configuration.set_is_worker_thread(true);
    interpreter_configuration.set_should_print_stack_on_signals(false);
    interpreter_configuration.set_extra_arguments(extra_args);
    Constellation::for_android(app).run(VirtualMachineConfiguration {
        interpreter_configuration,
        log_signals: None,
    });
    std::thread::sleep(Duration::from_secs(1));
}
