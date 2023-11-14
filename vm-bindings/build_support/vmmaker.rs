use std::fmt::{Display, Formatter};
use crate::{Builder, FamilyOS, HostOS, DOWNLOADING, EXTRACTING};
use anyhow::Result;
use commander::CommandToExecute;
use downloader::{FileToDownload, FilesToDownload};
use file_matcher::{FileNamed, OneEntryCopier};
use serde::{Serialize, Serializer};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;
use unzipper::{FileToUnzip, FilesToUnzip};

const VM_CLIENT_VMMAKER_VM_VAR: &str = "VM_CLIENT_VMMAKER";
const VM_CLIENT_VMMAKER_IMAGE_VAR: &str = "VM_CLIENT_VMMAKER_IMAGE";

const VMMAKER_LINUX_X86_64_VM_URL: VirtualMachineUrl = VirtualMachineUrl::GToolkit("https://github.com/feenkcom/gtoolkit-vm/releases/download/v0.6.5/GlamorousToolkit-x86_64-unknown-linux-gnu.zip");
const VMMAKER_LINUX_ARM64_VM_URL: VirtualMachineUrl = VirtualMachineUrl::Pharo("https://files.pharo.org/vm/pharo-spur64-headless/Linux-aarch64/PharoVM-9.0.11-9e68882-Linux-aarch64-bin.zip");

const VMMAKER_DARWIN_INTEL_VM_URL: VirtualMachineUrl = VirtualMachineUrl::GToolkit("https://github.com/feenkcom/gtoolkit-vm/releases/download/v0.6.5/GlamorousToolkit-x86_64-apple-darwin.app.zip");
const VMMAKER_DARWIN_M1_VM_URL: VirtualMachineUrl = VirtualMachineUrl::GToolkit(
    "https://github.com/feenkcom/gtoolkit-vm/releases/download/v0.6.5/GlamorousToolkit-aarch64-apple-darwin.app.zip",
);

const VMMAKER_WINDOWS_AMD64_VM_URL: VirtualMachineUrl = VirtualMachineUrl::GToolkit(
    "https://github.com/feenkcom/gtoolkit-vm/releases/download/v0.6.5/GlamorousToolkit-x86_64-pc-windows-msvc.zip",
);
const VMMAKER_WINDOWS_ARM64_VM_URL: VirtualMachineUrl = VirtualMachineUrl::GToolkit(
    "https://github.com/feenkcom/gtoolkit-vm/releases/download/v0.6.5/GlamorousToolkit-aarch64-pc-windows-msvc.zip",
);

const VMMAKER_IMAGE_URL: &str =
    "https://dl.feenk.com/gtvm/Pharo10.0.0-0.build.512.sha.bfb3a61.arch.64bit.zip";

/// a folder prefix within $OUT_DIR in which the vm is extracted
const VMMAKER_VM_FOLDER_PREFIX: &str = "vmmaker-vm";

#[derive(Debug, Clone, Serialize)]
pub struct VMMaker {
    vm: VirtualMachineExecutable,
    image: PathBuf,
    #[serde(skip)]
    builder: Rc<dyn Builder>,
}

#[derive(Debug, Clone)]
enum VirtualMachineUrl {
    Pharo(&'static str),
    GToolkit(&'static str),
}

impl VirtualMachineUrl {
    pub fn as_executable(&self) -> VirtualMachineExecutable {
        match self {
            Self::Pharo(_) => VirtualMachineExecutable::Pharo(PathBuf::new()),
            Self::GToolkit(_) => VirtualMachineExecutable::GToolkit(PathBuf::new()),
        }
    }
}

impl From<VirtualMachineUrl> for String {
    fn from(url: VirtualMachineUrl) -> Self {
        match url {
            VirtualMachineUrl::Pharo(url) => url.to_string(),
            VirtualMachineUrl::GToolkit(url) => url.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
enum VirtualMachineExecutable {
    Pharo(PathBuf),
    GToolkit(PathBuf),
}

impl VirtualMachineExecutable {
    fn all_by_priority() -> [Self; 2] {
        [Self::GToolkit(PathBuf::new()), Self::Pharo(PathBuf::new())]
    }

    fn name(&self) -> &'static str {
        match self {
            Self::Pharo(_) => "pharo",
            Self::GToolkit(_) => "gtoolkit",
        }
    }

    fn folder_name(&self) -> String {
        format!("{}-{}", VMMAKER_VM_FOLDER_PREFIX, self.name())
    }

    fn for_type(vm_type: impl AsRef<str>) -> Result<Self> {
        let vm_type = vm_type.as_ref().to_lowercase();
        match vm_type.as_str() {
            "pharo" => Ok(Self::Pharo(PathBuf::new())),
            "gtoolkit" => Ok(Self::GToolkit(PathBuf::new())),
            _ => bail!("Unsupported vm type: \"{}\"", vm_type),
        }
    }

    fn path(&self) -> &Path {
        match self {
            Self::Pharo(path) => path.as_path(),
            Self::GToolkit(path) => path.as_path(),
        }
    }

    fn exists(&self) -> bool {
        self.path().exists()
    }

    fn with_path(&self, path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();

        match self {
            Self::Pharo(_) => Self::Pharo(path),
            Self::GToolkit(_) => Self::GToolkit(path),
        }
    }

    fn with_host(&self, host: &HostOS, folder: impl AsRef<Path>) -> Self {
        let folder = folder.as_ref();
        match host.family() {
            FamilyOS::Apple => match self {
                Self::Pharo(_) => Self::Pharo(folder.join("Pharo.app/Contents/MacOS/Pharo")),
                Self::GToolkit(_) => Self::GToolkit(
                    folder.join("GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit-cli"),
                ),
            },
            FamilyOS::Unix | FamilyOS::Other => match self {
                Self::Pharo(_) => Self::Pharo(folder.join("pharo")),
                Self::GToolkit(_) => Self::GToolkit(folder.join("bin/GlamorousToolkit-cli")),
            },
            FamilyOS::Windows => match self {
                Self::Pharo(_) => Self::Pharo(folder.join("PharoConsole.exe")),
                Self::GToolkit(_) => {
                    Self::GToolkit(folder.join("bin").join("GlamorousToolkit-cli.exe"))
                }
            },
        }
    }

    pub fn as_command(&self) -> Command {
        let mut command = Command::new(self.path());
        match self {
            Self::Pharo(_) => {
                command.arg("--headless");
            }
            _ => {}
        };
        command
    }
}

struct VMMakerSource {
    vm: Option<VirtualMachineExecutable>,
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
        let custom_vmmaker_vm = Self::custom_vmmaker_vm()?;

        // if both the image and vm exist, we are done and can return the vmmaker
        if vmmaker_image.exists() && custom_vmmaker_vm.is_some() {
            return Ok(VMMaker {
                vm: custom_vmmaker_vm.clone().unwrap(),
                image: vmmaker_image,
                builder: builder.clone(),
            });
        }

        let existing_vmmaker_vms: Vec<VirtualMachineExecutable> =
            VirtualMachineExecutable::all_by_priority()
                .into_iter()
                .map(|each| {
                    each.with_host(
                        &builder.host(),
                        builder.output_directory().join(each.folder_name()),
                    )
                })
                .filter(|each| each.exists())
                .collect();

        // both vm and image exist, we are done
        if vmmaker_image.exists() && !existing_vmmaker_vms.is_empty() {
            return Ok(VMMaker {
                vm: existing_vmmaker_vms.first().unwrap().clone(),
                image: vmmaker_image,
                builder: builder.clone(),
            });
        }

        println!("VMMaker.image at {} exists: {}", vmmaker_image.display(), vmmaker_image.exists());
        println!("Custom VMMaker VM {:?}", &custom_vmmaker_vm);
        println!("Potential VMMaker VMs: {:?}", &existing_vmmaker_vms);

        // at this point we have neither a ready vmmaker image nor a vm.
        // we should download a new image if there is no custom one and get a vm
        let mut source = VMMakerSource {
            vm: Self::custom_vmmaker_vm()?,
            image: Self::custom_source_image(),
        };

        // if we have both source vm and the image there is nothing to download
        if source.vm.is_none() || source.image.is_none() {
            Self::download_vmmaker(&mut source, builder.clone())?;
        }

        let vmmaker_vm = source.vm.unwrap();
        let source_image = source.image.unwrap();

        CommandToExecute::build_command(vmmaker_vm.as_command(), |command| {
            command
                .arg(&source_image)
                .arg("save")
                .arg(vmmaker_image_dir.join("VMMaker"));
        })
        .with_name("Save image as VMMaker")
        .with_verbose(true)
        .without_log_prefix()
        .into_commands()
        .execute()?;

        FileNamed::wildmatch("*.sources")
            .within(source_image.parent().unwrap())
            .copy(&vmmaker_image_dir)?;

        CommandToExecute::build_command(vmmaker_vm.as_command(), |command| {
            command
                .arg(&vmmaker_image)
                .arg("st")
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
        })
        .with_name("Installing VMMaker")
        .with_verbose(true)
        .without_log_prefix()
        .into_commands()
        .execute()?;

        CommandToExecute::build_command(vmmaker_vm.as_command(), |command| {
            command
                .arg(&vmmaker_image)
                .arg("st")
                .arg("--no-default-preferences")
                .arg(
                    std::env::current_dir()
                        .unwrap()
                        .join("extra")
                        .join("vmmaker-patch.st"),
                );
        })
        .with_name("Patch VMMaker")
        .with_verbose(true)
        .without_log_prefix()
        .into_commands()
        .execute()?;

        return Ok(VMMaker {
            vm: vmmaker_vm,
            image: vmmaker_image,
            builder: builder.clone(),
        });
    }

    pub fn generate_sources(&self, interpreter: &str) -> Result<()> {
        if self.builder.output_directory().join("generated").exists() {
            return Ok(());
        }

        CommandToExecute::build_command(self.vm.as_command(), |command| {
            command.arg(&self.image).arg("eval").arg(format!(
                "PharoVMMaker generate: #'{}' outputDirectory: '{}' imageFormat: '{}'",
                interpreter,
                self.builder.output_directory().display(),
                self.builder.image_format()
            ));
        })
        .with_name("Generating sources")
        .with_verbose(true)
        .without_log_prefix()
        .into_commands()
        .execute()?;

        Ok(())
    }

    fn download_vmmaker(source: &mut VMMakerSource, builder: Rc<dyn Builder>) -> Result<()> {
        let url = match builder.host_family() {
            FamilyOS::Apple => match std::env::consts::ARCH {
                "aarch64" => VMMAKER_DARWIN_M1_VM_URL,
                "x86_64" => VMMAKER_DARWIN_INTEL_VM_URL,
                _ => bail!("Unsupported architecture: {}", std::env::consts::ARCH),
            },
            FamilyOS::Unix | FamilyOS::Other => match std::env::consts::ARCH {
                "aarch64" => VMMAKER_LINUX_ARM64_VM_URL,
                "x86_64" => VMMAKER_LINUX_X86_64_VM_URL,
                _ => bail!("Unsupported architecture: {}", std::env::consts::ARCH),
            },
            FamilyOS::Windows => match std::env::consts::ARCH {
                "aarch64" => VMMAKER_WINDOWS_ARM64_VM_URL,
                "x86_64" => VMMAKER_WINDOWS_AMD64_VM_URL,
                _ => bail!("Unsupported architecture: {}", std::env::consts::ARCH),
            },
        };

        let vm = FileToDownload::new(url.clone(), &builder.output_directory(), "vmmaker-vm.zip");
        let image = FileToDownload::new(
            VMMAKER_IMAGE_URL,
            &builder.output_directory(),
            "vmmaker-image.zip",
        );

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();

        let mut download = FilesToDownload::new();
        if source.vm.is_none() && !vm.already_downloaded() {
            download = download.add(vm.clone());
        }
        if source.image.is_none() && !image.already_downloaded() {
            download = download.add(image.clone());
        }

        if !download.is_empty() {
            println!("{}Downloading VMMaker", DOWNLOADING);
            rt.block_on(download.download())?;
        }

        let mut unzip = FilesToUnzip::new();

        let vm_executable = url.as_executable();

        let vm_folder = builder.output_directory().join(vm_executable.folder_name());
        let image_folder = builder.output_directory().join("vmmaker-image");

        if source.vm.is_none() && !vm_folder.exists() {
            unzip = unzip.add(FileToUnzip::new(vm.path(), &vm_folder));
        }
        if source.image.is_none() && !image_folder.exists() {
            unzip = unzip.add(FileToUnzip::new(image.path(), &image_folder));
        }
        if !unzip.is_empty() {
            println!("{}Extracting VMMaker", EXTRACTING);
            rt.block_on(unzip.unzip())?;
        }

        source.image = Some(
            FileNamed::wildmatch("*.image")
                .within(&image_folder)
                .find()?,
        );
        source.vm = Some(vm_executable.with_host(&builder.host(), vm_folder));
        Ok(())
    }

    fn custom_vmmaker_vm() -> Result<Option<VirtualMachineExecutable>> {
        std::env::var(VM_CLIENT_VMMAKER_VM_VAR).map_or(Ok(None), |value| {
            let type_and_path = value.split(":").collect::<Vec<&str>>();

            let executable = match type_and_path[..] {
                [] => bail!("The value of {} is empty", VM_CLIENT_VMMAKER_VM_VAR,),
                [path] => VirtualMachineExecutable::Pharo(Path::new(path).to_path_buf()),
                [vm_type, path] => VirtualMachineExecutable::for_type(vm_type)?.with_path(path),
                _ => bail!(
                    "The value of {} is malformed: {}",
                    VM_CLIENT_VMMAKER_VM_VAR,
                    &value
                ),
            };

            if executable.exists() {
                Ok(Some(executable))
            } else {
                bail!(
                    "Path specified in {} does not exist: {}",
                    VM_CLIENT_VMMAKER_VM_VAR,
                    executable.path().display()
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
}
