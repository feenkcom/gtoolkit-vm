import hudson.tasks.test.AbstractTestResultAction
import hudson.model.Actionable
import hudson.tasks.junit.CaseResult

pipeline {
    agent none
    options {
        buildDiscarder(logRotator(numToKeepStr: '50'))
        disableConcurrentBuilds()
    }
    environment {
        GITHUB_TOKEN = credentials('githubrelease')

        APP_NAME = 'GlamorousToolkit'
        APP_IDENTIFIER = 'com.gtoolkit'
        APP_LIBRARIES = 'git sdl2 boxer clipboard gleam glutin skia'
        APP_AUTHOR = '"feenk gmbh <contact@feenk.com>"'

        MACOS_INTEL_TARGET = 'x86_64-apple-darwin'
        MACOS_M1_TARGET = 'aarch64-apple-darwin'

        WINDOWS_SERVER_NAME = 'daffy-duck'
        WINDOWS_AMD64_TARGET = 'x86_64-pc-windows-msvc'

        LINUX_SERVER_NAME = 'mickey-mouse'
        LINUX_AMD64_TARGET = 'x86_64-unknown-linux-gnu'
    }

    stages {
        stage ('Read VM builder version') {
            agent {
                label "${MACOS_M1_TARGET}"
            }
            steps {
                script {
                    VM_BUILDER_VERSION = sh (
                        script: "cat vm-builder.version",
                        returnStdout: true
                    ).trim()
                }
                echo "Building using VM builder ${VM_BUILDER_VERSION}"
            }
        }
        stage ('Parallel build') {
            parallel {
                stage ('MacOS x86_64') {
                    agent {
                        label "${MACOS_INTEL_TARGET}"
                    }

                    environment {
                        TARGET = "${MACOS_INTEL_TARGET}"
                        PATH = "$HOME/.cargo/bin:/usr/local/bin/:$PATH"
                        CERT = credentials('devcertificate')
                        APPLEPASSWORD = credentials('notarizepassword')
                        MACOSX_DEPLOYMENT_TARGET = '10.10'
                    }

                    steps {
                        sh 'if [ -d target ]; then rm -Rf target; fi'
                        sh 'if [ -d third_party ]; then rm -Rf third_party; fi'
                        sh 'if [ -d libs ]; then rm -Rf third_party; fi'

                        sh 'git clean -fdx'
                        sh 'git submodule update --init --recursive'

                        sh "curl -o gtoolkit-vm-builder -LsS https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${TARGET}"
                        sh 'chmod +x gtoolkit-vm-builder'

                        sh """
                            ./gtoolkit-vm-builder \
                                --vmmaker-vm /Users/tudor/vmmaker/Pharo.app/Contents/MacOS/Pharo \
                                --vmmaker-image /Users/tudor/vmmaker/Pharo9.0-SNAPSHOT-64bit-ac4bf08.image \
                                --app-name ${APP_NAME} \
                                --identifier ${APP_IDENTIFIER} \
                                --author ${APP_AUTHOR} \
                                --icons icons/GlamorousToolkit.icns \
                                --libraries cairo freetype ${APP_LIBRARIES} \
                                --release """

                        sh "curl -o feenk-signer -LsS  https://github.com/feenkcom/feenk-signer/releases/latest/download/feenk-signer-${TARGET}"
                        sh "chmod +x feenk-signer"

                        sh "./feenk-signer target/${TARGET}/release/bundle/${APP_NAME}.app"

                        sh "ditto -c -k --sequesterRsrc --keepParent target/${TARGET}/release/bundle/${APP_NAME}.app ${APP_NAME}-${TARGET}.app.zip"
                        sh """
                           xcrun altool -t osx -f ${APP_NAME}-${TARGET}.app.zip -itc_provider "77664ZXL29" --primary-bundle-id "com.feenk.gtoolkit-vm-x86-64" --notarize-app --verbose  --username "george.ganea@feenk.com" --password "${APPLEPASSWORD}"
                           """
                        stash includes: "${APP_NAME}-${TARGET}.app.zip", name: "${TARGET}"
                    }
                }
                stage ('MacOS M1') {
                    agent {
                        label "${MACOS_M1_TARGET}"
                    }

                    environment {
                        TARGET = "${MACOS_M1_TARGET}"
                        PATH = "$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
                        CERT = credentials('devcertificate')
                        APPLEPASSWORD = credentials('notarizepassword')
                    }

                    steps {
                        sh 'if [ -d target ]; then rm -Rf target; fi'
                        sh 'if [ -d third_party ]; then rm -Rf third_party; fi'
                        sh 'if [ -d libs ]; then rm -Rf third_party; fi'
                        sh 'git clean -fdx'
                        sh 'git submodule update --init --recursive'

                        sh "curl -o gtoolkit-vm-builder -LsS https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${TARGET}"
                        sh 'chmod +x gtoolkit-vm-builder'

                        sh """
                            ./gtoolkit-vm-builder \
                                --vmmaker-vm /Users/tudor/vmmaker/Pharo.app/Contents/MacOS/Pharo \
                                --vmmaker-image /Users/tudor/vmmaker/Pharo9.0-SNAPSHOT-64bit-ac4bf08.image \
                                --app-name ${APP_NAME} \
                                --identifier ${APP_IDENTIFIER} \
                                --author ${APP_AUTHOR} \
                                --icons icons/GlamorousToolkit.icns \
                                --libraries cairo freetype ${APP_LIBRARIES} \
                                --release """

                        sh "curl -o feenk-signer -LsS  https://github.com/feenkcom/feenk-signer/releases/latest/download/feenk-signer-${TARGET}"
                        sh "chmod +x feenk-signer"

                        sh "./feenk-signer target/${TARGET}/release/bundle/${APP_NAME}.app"

                        sh "ditto -c -k --sequesterRsrc --keepParent target/${TARGET}/release/bundle/${APP_NAME}.app ${APP_NAME}-${TARGET}.app.zip"

                        sh """
                           xcrun altool -t osx -f ${APP_NAME}-${TARGET}.app.zip -itc_provider "77664ZXL29" --primary-bundle-id "com.feenk.gtoolkit-vm-aarch64" --notarize-app --verbose  --username "george.ganea@feenk.com" --password "${APPLEPASSWORD}"
                           """
                        stash includes: "${APP_NAME}-${TARGET}.app.zip", name: "${TARGET}"
                    }
                }
                stage ('Linux x86_64') {
                    agent {
                        label "${LINUX_AMD64_TARGET}-${LINUX_SERVER_NAME}"
                    }
                    environment {
                        TARGET = "${LINUX_AMD64_TARGET}"
                        PATH = "$HOME/.cargo/bin:$PATH"
                    }

                    steps {
                        sh 'if [ -d target ]; then rm -Rf target; fi'
                        sh 'if [ -d third_party ]; then rm -Rf third_party; fi'
                        sh 'if [ -d libs ]; then rm -Rf third_party; fi'
                        sh 'git clean -fdx'
                        sh 'git submodule update --init --recursive'

                        sh "curl -o gtoolkit-vm-builder -LsS https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${TARGET}"
                        sh 'chmod +x gtoolkit-vm-builder'

                        sh """
                            ./gtoolkit-vm-builder \
                                --vmmaker-vm /home/ubuntu/jenkins/vmmaker/pharo \
                                --vmmaker-image /home/ubuntu/jenkins/vmmaker/Pharo9.0-SNAPSHOT-64bit-ac4bf08.image \
                                --app-name ${APP_NAME} \
                                --identifier ${APP_IDENTIFIER} \
                                --author ${APP_AUTHOR} \
                                --libraries ${APP_LIBRARIES} \
                                --release """

                        sh """
                            cd target/${TARGET}/release/bundle/${APP_NAME}/
                            zip -r ${APP_NAME}-${TARGET}.zip .
                            """

                        sh 'mv target/${TARGET}/release/bundle/${APP_NAME}/${APP_NAME}-${TARGET}.zip ./${APP_NAME}-${TARGET}.zip'

                        stash includes: "${APP_NAME}-${TARGET}.zip", name: "${TARGET}"
                    }
                }
                stage ('Windows x86_64') {
                    agent {
                        node {
                          label "${WINDOWS_AMD64_TARGET}-${WINDOWS_SERVER_NAME}"
                          customWorkspace 'C:\\j\\gtvm'
                        }
                    }

                    environment {
                        TARGET = "${WINDOWS_AMD64_TARGET}"
                        MSVC_PATH = 'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\VC\\Tools\\MSVC\\14.29.30037\\bin\\Hostx64\\x64'
                        ASM_MASM = "${MSVC_PATH}\\ml64.exe"
                        LLVM_HOME = 'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\VC\\Tools\\Llvm\\x64'
                        LIBCLANG_PATH = "${LLVM_HOME}\\bin"
                        CMAKE_PATH = 'C:\\Program Files\\CMake\\bin'
                        MSBUILD_PATH = 'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\MSBuild\\Current\\Bin'
                        CARGO_HOME = "C:\\.cargo"
                        CARGO_PATH = "${CARGO_HOME}\\bin"
                        PATH = "${CARGO_PATH};${LIBCLANG_PATH};${MSBUILD_PATH};${CMAKE_PATH};${MSVC_PATH};$PATH"
                    }

                    steps {
                        powershell 'Remove-Item -Force -Recurse -Path target -ErrorAction Ignore'
                        powershell 'Remove-Item -Force -Recurse -Path third_party -ErrorAction Ignore'
                        powershell 'Remove-Item -Force -Recurse -Path libs -ErrorAction Ignore'
                        powershell 'git clean -fdx'
                        powershell 'git submodule update --init --recursive'

                        powershell "curl -o gtoolkit-vm-builder.exe https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${TARGET}.exe"

                        powershell """
                           ./gtoolkit-vm-builder.exe `
                                --vmmaker-vm C:/j/vmmaker/PharoConsole.exe `
                                --vmmaker-image C:/j/vmmaker/Pharo9.0-SNAPSHOT-64bit-ac4bf08.image `
                                --app-name ${APP_NAME} `
                                --identifier ${APP_IDENTIFIER} `
                                --author ${APP_AUTHOR} `
                                --libraries cairo freetype ${APP_LIBRARIES} `
                                --icons icons/GlamorousToolkit.ico `
                                --release """

                        powershell "Compress-Archive -Path target/${TARGET}/release/bundle/${APP_NAME}/* -DestinationPath ${APP_NAME}-${TARGET}.zip"
                        stash includes: "${APP_NAME}-${TARGET}.zip", name: "${TARGET}"
                    }
                }
            }
        }

        stage ('Deployment') {
            agent {
                label "${LINUX_AMD64_TARGET}-${LINUX_SERVER_NAME}"
            }
            environment {
                TARGET = "${LINUX_AMD64_TARGET}"
            }
            when {
                expression {
                    (currentBuild.result == null || currentBuild.result == 'SUCCESS') && env.BRANCH_NAME.toString().equals('main')
                }
            }
            steps {
                unstash "${LINUX_AMD64_TARGET}"
                unstash "${MACOS_INTEL_TARGET}"
                unstash "${MACOS_M1_TARGET}"
                unstash "${WINDOWS_AMD64_TARGET}"

                sh "wget -O feenk-releaser https://github.com/feenkcom/releaser-rs/releases/latest/download/feenk-releaser-${TARGET}"
                sh "chmod +x feenk-releaser"

                sh """
                ./feenk-releaser \
                    --owner feenkcom \
                    --repo gtoolkit-vm \
                    --token GITHUB_TOKEN \
                    --bump-patch \
                    --auto-accept \
                    --assets \
                        ${APP_NAME}-${LINUX_AMD64_TARGET}.zip \
                        ${APP_NAME}-${MACOS_INTEL_TARGET}.app.zip \
                        ${APP_NAME}-${MACOS_M1_TARGET}.app.zip \
                        ${APP_NAME}-${WINDOWS_AMD64_TARGET}.zip """
            }
        }
    }
}
