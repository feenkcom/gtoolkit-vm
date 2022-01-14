use crate::Builder;
use bindgen::builder;
use file_matcher::{FileNamed, OneEntryCopier};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const VM_CLIENT_VMMAKER_VM_VAR: &str = "VM_CLIENT_VMMAKER";
const VM_CLIENT_VMMAKER_IMAGE_VAR: &str = "VM_CLIENT_VMMAKER_IMAGE";

const VMMAKER_WINDOWS_VM_URL: &str = "https://files.pharo.org/vm/pharo-spur64-headless/Windows-x86_64/PharoVM-9.0.11-9e688828-Windows-x86_64-bin.zip";
const VMMAKER_LINUX_VM_URL: &str = "https://files.pharo.org/vm/pharo-spur64-headless/Linux-x86_64/PharoVM-9.0.11-9e68882-Linux-x86_64-bin.zip";
const VMMAKER_DARWIN_VM_URL: &str = "https://files.pharo.org/vm/pharo-spur64-headless/Darwin-x86_64/PharoVM-9.0.11-9e688828-Darwin-x86_64-bin.zip";
const VMMAKER_IMAGE_URL: &str =
    "https://files.pharo.org/image/100/Pharo10-SNAPSHOT.build.349.sha.3e26baf.arch.64bit.zip";

enum VMMakerVM {}

pub struct VMMaker {
    vm: PathBuf,
    image: PathBuf,
}

impl VMMaker {
    pub fn prepare(builder: &Box<dyn Builder>) -> Self {
        let vm = Self::vmmaker_vm().unwrap();
        let source_image = Self::vmmaker_image().unwrap();

        let vmmaker_dir = builder.output_directory().join("vmmaker");
        if !vmmaker_dir.exists() {
            std::fs::create_dir_all(&vmmaker_dir).unwrap();
        };

        let vmmaker_image = vmmaker_dir.join("VMMaker.image");

        let vmmaker = VMMaker {
            vm: vm.clone(),
            image: vmmaker_image.clone(),
        };

        if vmmaker_image.exists() {
            return vmmaker;
        }

        let status = Command::new(&vm)
            .arg("--headless")
            .arg(&source_image)
            .arg("save")
            .arg(vmmaker_dir.join("VMMaker"))
            .status()
            .unwrap();

        if !status.success() {
            panic!("Failed to create VMMaker image");
        }

        FileNamed::wildmatch("*.sources")
            .within(source_image.parent().unwrap())
            .copy(&vmmaker_dir)
            .expect("Copy the .sources");

        println!("Loading VMMaker...");
        let mut child = Command::new(&vm)
            .stdout(Stdio::piped())
            .arg("--headless")
            .arg(&vmmaker_image)
            .arg("--no-default-preferences")
            .arg("--save")
            .arg("--quit")
            .arg(
                builder
                    .vm_sources_directory()
                    .join("scripts")
                    .join("installVMMaker.st"),
            )
            .arg(builder.vm_sources_directory())
            .arg("scpUrl")
            .spawn()
            .unwrap();

        let stdout = (&mut child).stdout.take().unwrap();

        let reader = BufReader::new(stdout);
        reader
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| println!("{}", line));

        if !child.wait().unwrap().success() {
            panic!("Failed to install VMMaker");
        }

        vmmaker
    }

    pub fn generate_sources(&self, builder: &Box<dyn Builder>) {
        if builder.output_directory().join("generated").exists() {
            return;
        }

        println!("Generating sources...");
        let mut command = Command::new(&self.vm);
        command
            .stdout(Stdio::piped())
            .arg("--headless")
            .arg(&self.image)
            .arg("--no-default-preferences")
            .arg("eval")
            .arg(format!(
                "PharoVMMaker generate: #'{}' outputDirectory: '{}'",
                "CoInterpreter",
                builder.output_directory().display()
            ));
        println!("{:?}", &command);
        let mut child = command.spawn().unwrap();

        let stdout = (&mut child).stdout.take().unwrap();

        let reader = BufReader::new(stdout);
        reader
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| println!("{}", line));

        if !child.wait().unwrap().success() {
            panic!("Failed to generate sources");
        }
    }

    fn vmmaker_vm() -> Option<PathBuf> {
        std::env::var(VM_CLIENT_VMMAKER_VM_VAR).map_or(None, |path| {
            let path = Path::new(&path);
            if path.exists() {
                Some(path.to_path_buf())
            } else {
                panic!(
                    "Specified {} does not exist: {}",
                    VM_CLIENT_VMMAKER_VM_VAR,
                    path.display()
                );
            }
        })
    }

    fn vmmaker_image() -> Option<PathBuf> {
        std::env::var(VM_CLIENT_VMMAKER_IMAGE_VAR).map_or(None, |path| {
            let path = Path::new(&path);
            if path.exists() {
                Some(path.to_path_buf())
            } else {
                panic!(
                    "Specified {} does not exist: {}",
                    VM_CLIENT_VMMAKER_IMAGE_VAR,
                    path.display()
                );
            }
        })
    }
}
