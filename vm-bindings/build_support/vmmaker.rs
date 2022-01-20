use crate::{Builder, BuilderTarget, DOWNLOADING, EXTRACTING};
use anyhow::{anyhow, Result};
use commander::{CommandToExecute, CommandsToExecute};
use downloader::{FileToDownload, FilesToDownload};
use file_matcher::{FileNamed, OneEntryCopier};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::rc::Rc;
use unzipper::{FileToUnzip, FilesToUnzip};

const VM_CLIENT_VMMAKER_VM_VAR: &str = "VM_CLIENT_VMMAKER";
const VM_CLIENT_VMMAKER_IMAGE_VAR: &str = "VM_CLIENT_VMMAKER_IMAGE";

const VMMAKER_WINDOWS_VM_URL: &str = "https://files.pharo.org/vm/pharo-spur64-headless/Windows-x86_64/PharoVM-9.0.11-9e688828-Windows-x86_64-bin.zip";
const VMMAKER_LINUX_VM_URL: &str = "https://files.pharo.org/vm/pharo-spur64-headless/Linux-x86_64/PharoVM-9.0.11-9e68882-Linux-x86_64-bin.zip";
const VMMAKER_DARWIN_VM_URL: &str = "https://files.pharo.org/vm/pharo-spur64-headless/Darwin-x86_64/PharoVM-9.0.11-9e688828-Darwin-x86_64-bin.zip";
const VMMAKER_IMAGE_URL: &str =
    "https://files.pharo.org/image/90/Pharo9.0-SNAPSHOT.build.1574.sha.6f28d0a.arch.64bit.zip";

/// a folder within $OUT_DIR in which the vm is extracted
const VMMAKER_VM_FOLDER: &str = "vmmaker-vm";

#[derive(Debug, Clone)]
pub struct VMMaker {
    vm: PathBuf,
    image: PathBuf,
    builder: Rc<dyn Builder>,
}

struct VMMakerSource {
    vm: Option<PathBuf>,
    image: Option<PathBuf>,
}

impl VMMaker {
    pub fn prepare(builder: Rc<dyn Builder>) -> Result<Self> {
        // a directory in which the vmmaker image will be created
        let vmmaker_image_dir = builder.output_directory().join("vmmaker");
        if !vmmaker_image_dir.exists() {
            std::fs::create_dir_all(&vmmaker_image_dir)?;
        };

        // the definitive location of the vmmaker image
        let vmmaker_image = vmmaker_image_dir.join("VMMaker.image");
        // let's see if the vm is provided by the user
        let vmmaker_vm = Self::custom_vmmaker_vm();

        // if both the image and vm exist, we are done and can return the vmmaker
        if vmmaker_image.exists() && vmmaker_vm.is_some() {
            return Ok(VMMaker {
                vm: vmmaker_vm.unwrap(),
                image: vmmaker_image,
                builder: builder.clone(),
            });
        }

        let vmmaker_vm_dir = builder.output_directory().join(VMMAKER_VM_FOLDER);
        // an expected location of the downloaded and extracted vm
        let vmmaker_vm = Self::vmmaker_executable(builder.clone(), vmmaker_vm_dir.clone());

        // both vm and image exist, we are done
        if vmmaker_image.exists() && vmmaker_vm.exists() {
            return Ok(VMMaker {
                vm: vmmaker_vm,
                image: vmmaker_image,
                builder: builder.clone(),
            });
        }

        // at this point we have neither a ready vmmaker image nor a vm.
        // we should download a new image if there is no custom one and get a vm
        let mut source = VMMakerSource {
            vm: Self::custom_vmmaker_vm(),
            image: Self::custom_source_image(),
        };

        // if we have both source vm and the image there is nothing to download
        if source.vm.is_none() || source.image.is_none() {
            Self::download_vmmaker(&mut source, builder.clone())?;
        }

        let vmmaker_vm = source.vm.unwrap();
        let source_image = source.image.unwrap();

        let status = Command::new(&vmmaker_vm)
            .arg("--headless")
            .arg(&source_image)
            .arg("save")
            .arg(vmmaker_image_dir.join("VMMaker"))
            .status()?;

        if !status.success() {
            anyhow!("Failed to create VMMaker image");
        }

        FileNamed::wildmatch("*.sources")
            .within(source_image.parent().unwrap())
            .copy(&vmmaker_image_dir)?;

        let mut command = Command::new(&vmmaker_vm);
        command
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
            .arg("scpUrl");

        CommandsToExecute::new()
            .add(
                CommandToExecute::new(command)
                    .with_name("Installing VMMaker")
                    .with_verbose(true)
                    .without_log_prefix(),
            )
            .execute()?;

        return Ok(VMMaker {
            vm: vmmaker_vm,
            image: vmmaker_image,
            builder: builder.clone(),
        });
    }

    pub fn generate_sources(&self) {
        if self.builder.output_directory().join("generated").exists() {
            return;
        }

        let mut command = Command::new(&self.vm);
        command
            .arg("--headless")
            .arg(&self.image)
            .arg("--no-default-preferences")
            .arg("eval")
            .arg(format!(
                "PharoVMMaker generate: #'{}' outputDirectory: '{}'",
                "CoInterpreter",
                self.builder.output_directory().display()
            ));

        CommandsToExecute::new()
            .add(
                CommandToExecute::new(command)
                    .with_name("Generating sources")
                    .with_verbose(true)
                    .without_log_prefix(),
            )
            .execute()
            .unwrap();
    }

    fn download_vmmaker(source: &mut VMMakerSource, builder: Rc<dyn Builder>) -> Result<()> {
        let url = match builder.target() {
            BuilderTarget::MacOS => VMMAKER_DARWIN_VM_URL,
            BuilderTarget::Linux => VMMAKER_LINUX_VM_URL,
            BuilderTarget::Windows => VMMAKER_WINDOWS_VM_URL,
        };

        let vm = FileToDownload::new(url, &builder.output_directory(), "vmmaker-vm.zip");
        let image = FileToDownload::new(
            VMMAKER_IMAGE_URL,
            &builder.output_directory(),
            "vmmaker-image.zip",
        );

        let mut download = FilesToDownload::new();
        if source.vm.is_none() {
            download = download.add(vm.clone());
        }
        if source.image.is_none() {
            download = download.add(image.clone());
        }

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();

        println!("{:?}", &download);
        println!("{}Downloading VMMaker", DOWNLOADING);
        rt.block_on(download.download())?;

        let mut unzip = FilesToUnzip::new();

        let vm_folder = builder.output_directory().join(VMMAKER_VM_FOLDER);
        let image_folder = builder.output_directory().join("vmmaker-image");

        if source.vm.is_none() {
            unzip = unzip.add(FileToUnzip::new(vm.path(), &vm_folder));
        }
        if source.image.is_none() {
            unzip = unzip.add(FileToUnzip::new(image.path(), &image_folder));
        }

        println!("{}Extracting VMMaker", EXTRACTING);
        rt.block_on(unzip.unzip())?;

        source.image = Some(
            FileNamed::wildmatch("*.image")
                .within(&image_folder)
                .find()?,
        );
        source.vm = Some(Self::vmmaker_executable(builder.clone(), vm_folder));
        Ok(())
    }

    fn custom_vmmaker_vm() -> Option<PathBuf> {
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

    /// Return a path to the source image that should be used to create a vmmaker
    fn custom_source_image() -> Option<PathBuf> {
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

    fn vmmaker_executable(builder: Rc<dyn Builder>, vm_folder: PathBuf) -> PathBuf {
        match builder.target() {
            BuilderTarget::MacOS => vm_folder
                .join("Pharo.app")
                .join("Contents")
                .join("MacOS")
                .join("Pharo"),
            BuilderTarget::Linux => vm_folder.join("pharo"),
            BuilderTarget::Windows => vm_folder.join("PharoConsole.exe"),
        }
    }
}
