use std::env;
use std::ffi::CString;
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;
use std::time::Duration;
use log::info;
use vm_runtime::vm_bindings::InterpreterConfiguration;
use vm_runtime::{android_activity, Constellation, VirtualMachineConfiguration};
use zip::ZipArchive;

fn print_dir_tree(path: &Path, indent: usize) {
    let prefix = " ".repeat(indent);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_else(|| path.display().to_string().into());
    println!("{}{}", prefix, name);

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(err) => {
            println!("{}  <error: {}>", prefix, err);
            return;
        }
    };

    let mut entries: Vec<_> = entries.filter_map(|entry| entry.ok()).collect();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let entry_path = entry.path();
        match entry.file_type() {
            Ok(file_type) if file_type.is_dir() => print_dir_tree(&entry_path, indent + 2),
            Ok(_) => println!("{}  {}", prefix, entry.file_name().to_string_lossy()),
            Err(err) => println!(
                "{}  {} <type error: {}>",
                prefix,
                entry.file_name().to_string_lossy(),
                err
            ),
        }
    }
}

fn unzip_to_directory(archive_path: &Path, destination: &Path) -> io::Result<()> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)
        .map_err(|err| io::Error::new(io::ErrorKind::Other, format!("zip open: {err}")))?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let Some(relative_path) = file.enclosed_name().map(|p| p.to_owned()) else {
            continue;
        };

        let out_path = destination.join(relative_path);

        if file.is_dir() {
            fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = fs::File::create(&out_path)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    Ok(())
}

#[no_mangle]
pub extern "C" fn android_main(app: android_activity::AndroidApp) {
    env::set_var("RUST_LOG", "trace");
    env::set_var("RUST_BACKTRACE", "full");

    std::thread::sleep(Duration::from_secs(5));
    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Trace),
    );

    let external_path = app.external_data_path().expect("Get external data path");
    println!("external_path: {}", external_path.display());

    let asset_manager = app.asset_manager();
    let glamorous_toolkit_archive_name = "GlamorousToolkit.zip";
    let copied_glamorous_toolkit_archive = external_path.join(glamorous_toolkit_archive_name);
    let image_path = external_path.join("GlamorousToolkit.image");
    
    if !copied_glamorous_toolkit_archive.exists() {
        if let Some(mut asset) = asset_manager.open(&CString::new(glamorous_toolkit_archive_name).unwrap()) {
            let mut output = fs::File::create(external_path.join(glamorous_toolkit_archive_name)).unwrap();
            io::copy(&mut asset, &mut output).unwrap();
            
        } else {
            panic!("Could not find GlamorousToolkit.zip in assets directory");
        }
    }

    if !image_path.exists() {
        println!(
            "Extracting {} into {}",
            glamorous_toolkit_archive_name,
            external_path.display()
        );
        unzip_to_directory(&copied_glamorous_toolkit_archive, &external_path).unwrap_or_else(|err| {
            panic!(
                "Failed to unzip {} into {}: {}",
                copied_glamorous_toolkit_archive.display(),
                external_path.display(),
                err
            )
        });
    } else {
        println!(
            "Archive already extracted; found image at {}",
            image_path.display()
        );
    }

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

    let extra_args = vec![];

    let mut interpreter_configuration = InterpreterConfiguration::new(image_path);
    interpreter_configuration.set_interactive_session(true);
    interpreter_configuration.set_is_worker_thread(true);
    interpreter_configuration.set_should_print_stack_on_signals(false);
    interpreter_configuration.set_extra_arguments(extra_args);
    
    info!("AndroidNativeWindow: {:?}", app.native_window());
    
    Constellation::for_android(app).run(VirtualMachineConfiguration {
        interpreter_configuration,
        log_signals: Some(vec![]),
    });
    std::thread::sleep(Duration::from_secs(1));
}
