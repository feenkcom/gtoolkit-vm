import hudson.tasks.test.AbstractTestResultAction
import hudson.model.Actionable
import hudson.tasks.junit.CaseResult

pipeline {
    agent none
    parameters {
        choice(name: 'BUMP', choices: ['patch', 'minor', 'major'], description: 'What to bump when releasing') }
    options {
        buildDiscarder(logRotator(numToKeepStr: '50'))
        disableConcurrentBuilds()
    }
    environment {
        GITHUB_TOKEN = credentials('githubrelease')

        REPOSITORY_OWNER = 'feenkcom'
        REPOSITORY_NAME = 'gtoolkit-vm'

        APP_NAME = 'GlamorousToolkit'
        APP_IDENTIFIER = 'com.gtoolkit'
        APP_LIBRARIES = 'boxer clipboard filewatcher gleam glutin pixels process skia winit webview test-library cairo crypto freetype git sdl2 ssl'
        LINUX_APP_LIBRARIES_AMD = 'boxer clipboard filewatcher gleam glutin pixels process skia webview winit test-library cairo crypto freetype git sdl2 ssl'
        LINUX_APP_LIBRARIES_ARM = '      clipboard filewatcher gleam glutin pixels process skia webview winit test-library       crypto freetype git      ssl'
        APP_LIBRARIES_VERSIONS = 'libraries.version'
        APP_AUTHOR = '"feenk gmbh <contact@feenk.com>"'

        MACOS_INTEL_TARGET = 'x86_64-apple-darwin'
        MACOS_M1_TARGET = 'aarch64-apple-darwin'

        WINDOWS_AMD64_SERVER_NAME = 'daffy-duck'
        WINDOWS_AMD64_TARGET = 'x86_64-pc-windows-msvc'
        WINDOWS_ARM64_SERVER_NAME = 'bugs-bunny'
        WINDOWS_ARM64_TARGET = 'aarch64-pc-windows-msvc'

        LINUX_AMD64_SERVER_NAME = 'mickey-mouse'
        LINUX_AMD64_TARGET = 'x86_64-unknown-linux-gnu'
        LINUX_ARM64_SERVER_NAME = 'peter-pan'
        LINUX_ARM64_TARGET = 'aarch64-unknown-linux-gnu'

        ANDROID_ARM64_TARGET = 'aarch64-linux-android'
    }

    stages {
        stage ('Read tool versions') {
            agent {
                label "${MACOS_M1_TARGET}"
            }
            environment {
                TARGET = "${MACOS_M1_TARGET}"
            }
            steps {
                script {
                    VM_BUILDER_VERSION = sh (
                        script: "cat vm-builder.version",
                        returnStdout: true
                    ).trim()
                    FEENK_RELEASER_VERSION = sh (
                        script: "cat feenk-releaser.version",
                        returnStdout: true
                    ).trim()
                    FEENK_SIGNER_VERSION = sh (
                        script: "cat feenk-signer.version",
                        returnStdout: true
                    ).trim()
                }

                sh "rm -rf feenk-releaser"
                sh "curl -o feenk-releaser -LsS https://github.com/feenkcom/releaser-rs/releases/download/${FEENK_RELEASER_VERSION}/feenk-releaser-${TARGET}"
                sh "chmod +x feenk-releaser"
                script {
                    APP_VERSION = sh (
                        script: "./feenk-releaser --owner ${REPOSITORY_OWNER} --repo ${REPOSITORY_NAME} --token GITHUB_TOKEN next-version --bump ${params.BUMP}",
                        returnStdout: true
                    ).trim()
                }
                echo "Will build using gtoolkit-vm-builder ${VM_BUILDER_VERSION}"
                echo "Will release using feenk-releaser ${FEENK_RELEASER_VERSION}"
                echo "Will sign using feenk-signer ${FEENK_SIGNER_VERSION}"
                echo "Will release a new version: ${APP_VERSION}"
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
                        APPLEPASSWORD = credentials('notarizepassword-manager')
                        MACOSX_DEPLOYMENT_TARGET = '10.10'
                        VM_CLIENT_EXECUTABLE = "${WORKSPACE}/target/${TARGET}/release/bundle/${APP_NAME}.app/Contents/MacOS/${APP_NAME}-cli"
                    }

                    steps {
                        sh 'if [ -d target ]; then rm -Rf target; fi'
                        sh 'if [ -d third_party ]; then rm -Rf third_party; fi'
                        sh 'if [ -d libs ]; then rm -Rf libs; fi'

                        sh 'git clean -fdx'
                        sh 'git submodule foreach --recursive \'git fetch --tags\''
                        sh 'git submodule update --init --recursive'

                        sh "rm -rf gtoolkit-vm-builder"
                        sh "curl -o gtoolkit-vm-builder -LsS https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${TARGET}"
                        sh 'chmod +x gtoolkit-vm-builder'

                        sh """
                            ./gtoolkit-vm-builder \
                                --app-name ${APP_NAME} \
                                --identifier ${APP_IDENTIFIER} \
                                --author ${APP_AUTHOR} \
                                --version ${APP_VERSION} \
                                --icons icons/GlamorousToolkit.icns \
                                --libraries ${APP_LIBRARIES} \
                                --libraries-versions ${APP_LIBRARIES_VERSIONS} \
                                --release \
                                --verbose """

                        sh "curl -o feenk-signer -LsS https://github.com/feenkcom/feenk-signer/releases/download/${FEENK_SIGNER_VERSION}/feenk-signer-${TARGET}"
                        sh "chmod +x feenk-signer"

                        withCredentials([file(credentialsId: 'feenk-apple-developer-certificate', variable: 'CERT')]) {
                            sh "./feenk-signer mac target/${TARGET}/release/bundle/${APP_NAME}.app"
                        }

                        sh "ditto -c -k --sequesterRsrc --keepParent target/${TARGET}/release/bundle/${APP_NAME}.app ${APP_NAME}-${TARGET}.app.zip"

                        sh "cargo test --package vm-client-tests"

                        sh """
                           /Library/Developer/CommandLineTools/usr/bin/notarytool submit \
                                --verbose \
                                --apple-id "notarization@feenk.com" \
                                --password "${APPLEPASSWORD}" \
                                --team-id "77664ZXL29" \
                                --wait \
                                ${APP_NAME}-${TARGET}.app.zip
                           """

//                        sh """
//                           xcrun altool -t osx -f ${APP_NAME}-${TARGET}.app.zip -itc_provider "77664ZXL29" --primary-bundle-id "com.feenk.gtoolkit-vm-x86-64" --notarize-app --verbose  --username "notarization@feenk.com" --password "${APPLEPASSWORD}"
//                           """
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
                        APPLEPASSWORD = credentials('notarizepassword-manager')
                        VM_CLIENT_EXECUTABLE = "${WORKSPACE}/target/${TARGET}/release/bundle/${APP_NAME}.app/Contents/MacOS/${APP_NAME}-cli"
                    }

                    steps {
                        sh 'if [ -d target ]; then rm -Rf target; fi'
                        sh 'if [ -d third_party ]; then rm -Rf third_party; fi'
                        sh 'if [ -d libs ]; then rm -Rf libs; fi'

                        sh 'git clean -fdx'
                        sh 'git submodule foreach --recursive \'git fetch --tags\''
                        sh 'git submodule update --init --recursive'

                        sh "rm -rf gtoolkit-vm-builder"
                        sh "curl -o gtoolkit-vm-builder -LsS https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${TARGET}"
                        sh 'chmod +x gtoolkit-vm-builder'

                        sh """
                            ./gtoolkit-vm-builder \
                                --app-name ${APP_NAME} \
                                --identifier ${APP_IDENTIFIER} \
                                --author ${APP_AUTHOR} \
                                --version ${APP_VERSION} \
                                --icons icons/GlamorousToolkit.icns \
                                --libraries ${APP_LIBRARIES} \
                                --libraries-versions ${APP_LIBRARIES_VERSIONS} \
                                --release \
                                --verbose """

                        sh "curl -o feenk-signer -LsS  https://github.com/feenkcom/feenk-signer/releases/download/${FEENK_SIGNER_VERSION}/feenk-signer-${TARGET}"
                        sh "chmod +x feenk-signer"

                        withCredentials([file(credentialsId: 'feenk-apple-developer-certificate', variable: 'CERT')]) {
                            sh "./feenk-signer mac target/${TARGET}/release/bundle/${APP_NAME}.app"
                        }

                        sh "ditto -c -k --sequesterRsrc --keepParent target/${TARGET}/release/bundle/${APP_NAME}.app ${APP_NAME}-${TARGET}.app.zip"

                        sh "cargo test --package vm-client-tests"

                        sh """
                           /Library/Developer/CommandLineTools/usr/bin/notarytool submit \
                                --verbose \
                                --apple-id "notarization@feenk.com" \
                                --password "${APPLEPASSWORD}" \
                                --team-id "77664ZXL29" \
                                --wait \
                                ${APP_NAME}-${TARGET}.app.zip
                           """

//                        sh """
//                           xcrun altool -t osx -f ${APP_NAME}-${TARGET}.app.zip -itc_provider "77664ZXL29" --primary-bundle-id "com.feenk.gtoolkit-vm-aarch64" --notarize-app --verbose  --username "notarization@feenk.com" --password "${APPLEPASSWORD}"
//                           """
                        stash includes: "${APP_NAME}-${TARGET}.app.zip", name: "${TARGET}"
                    }
                }
                stage ('Linux x86_64') {
                    agent {
                        label "${LINUX_AMD64_TARGET}-${LINUX_AMD64_SERVER_NAME}"
                    }
                    environment {
                        TARGET = "${LINUX_AMD64_TARGET}"
                        PATH = "$HOME/.cargo/bin:$HOME/patchelf/bin:$PATH"
                        OPENSSL_STATIC = 1
                        OPENSSL_LIB_DIR = "/usr/lib/x86_64-linux-gnu"
                        OPENSSL_INCLUDE_DIR = "/usr/include/openssl"
                        VM_CLIENT_EXECUTABLE = "${WORKSPACE}/target/${TARGET}/release/bundle/${APP_NAME}/bin/${APP_NAME}-cli"
                    }

                    steps {
                        sh 'if [ -d target ]; then rm -Rf target; fi'
                        sh 'if [ -d third_party ]; then rm -Rf third_party; fi'
                        sh 'if [ -d libs ]; then rm -Rf libs; fi'

                        sh 'git clean -fdx'
                        sh 'git submodule foreach --recursive \'git fetch --tags\''
                        sh 'git submodule update --init --recursive'

                        sh "rm -rf gtoolkit-vm-builder"
                        sh "curl -o gtoolkit-vm-builder -LsS https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${TARGET}"
                        sh 'chmod +x gtoolkit-vm-builder'
                        sh 'echo "patchelf $(patchelf --version)"'

                        sh """
                            ./gtoolkit-vm-builder \
                                --app-name ${APP_NAME} \
                                --identifier ${APP_IDENTIFIER} \
                                --author ${APP_AUTHOR} \
                                --version ${APP_VERSION} \
                                --libraries ${LINUX_APP_LIBRARIES_AMD} \
                                --libraries-versions ${APP_LIBRARIES_VERSIONS} \
                                --release \
                                --verbose """

                        sh """
                            cd target/${TARGET}/release/bundle/${APP_NAME}/
                            zip -r ${APP_NAME}-${TARGET}.zip .
                            """

                        sh "cargo test --package vm-client-tests"

                        sh 'mv target/${TARGET}/release/bundle/${APP_NAME}/${APP_NAME}-${TARGET}.zip ./${APP_NAME}-${TARGET}.zip'

                        stash includes: "${APP_NAME}-${TARGET}.zip", name: "${TARGET}"
                    }
                }
                stage ('Linux arm64') {
                    agent {
                        label "${LINUX_ARM64_TARGET}-${LINUX_ARM64_SERVER_NAME}"
                    }
                    environment {
                        TARGET = "${LINUX_ARM64_TARGET}"
                        PATH = "$HOME/.cargo/bin:$HOME/patchelf/bin:$PATH"
                        OPENSSL_STATIC = 1
                        OPENSSL_LIB_DIR = "/usr/lib/aarch64-linux-gnu"
                        OPENSSL_INCLUDE_DIR = "/usr/include/openssl"
                        VM_CLIENT_EXECUTABLE = "${WORKSPACE}/target/${TARGET}/release/bundle/${APP_NAME}/bin/${APP_NAME}-cli"
                    }

                    steps {
                        sh 'if [ -d target ]; then rm -Rf target; fi'
                        sh 'if [ -d third_party ]; then rm -Rf third_party; fi'
                        sh 'if [ -d libs ]; then rm -Rf libs; fi'

                        sh 'git clean -fdx'
                        sh 'git submodule foreach --recursive \'git fetch --tags\''
                        sh 'git submodule update --init --recursive'

                        sh "rm -rf gtoolkit-vm-builder"
                        sh "curl -o gtoolkit-vm-builder -LsS https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${TARGET}"
                        sh 'chmod +x gtoolkit-vm-builder'

                        sh """
                            ./gtoolkit-vm-builder \
                                --app-name ${APP_NAME} \
                                --identifier ${APP_IDENTIFIER} \
                                --author ${APP_AUTHOR} \
                                --version ${APP_VERSION} \
                                --libraries ${LINUX_APP_LIBRARIES_ARM} \
                                --libraries-versions ${APP_LIBRARIES_VERSIONS} \
                                --release \
                                --verbose """

                        sh """
                            cd target/${TARGET}/release/bundle/${APP_NAME}/
                            zip -r ${APP_NAME}-${TARGET}.zip .
                            """

                        sh "cargo test --package vm-client-tests"

                        sh 'mv target/${TARGET}/release/bundle/${APP_NAME}/${APP_NAME}-${TARGET}.zip ./${APP_NAME}-${TARGET}.zip'

                        stash includes: "${APP_NAME}-${TARGET}.zip", name: "${TARGET}"
                    }
                }
                stage ('Android arm64') {
                    agent {
                        label "${MACOS_M1_TARGET}"
                    }
                    environment {
                        HOST = "${MACOS_M1_TARGET}"
                        TARGET = "${ANDROID_ARM64_TARGET}"
                        PATH = "$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
                    }

                    steps {
                        sh 'if [ -d target ]; then rm -Rf target; fi'
                        sh 'if [ -d third_party ]; then rm -Rf third_party; fi'
                        sh 'if [ -d libs ]; then rm -Rf libs; fi'

                        sh 'git clean -fdx'
                        sh 'git submodule foreach --recursive \'git fetch --tags\''
                        sh 'git submodule update --init --recursive'

                        sh "rm -rf gtoolkit-vm-builder"
                        sh "curl -o gtoolkit-vm-builder -LsS https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${HOST}"
                        sh 'chmod +x gtoolkit-vm-builder'

                        sh """
                            ./gtoolkit-vm-builder \
                                --app-name ${APP_NAME} \
                                --identifier ${APP_IDENTIFIER} \
                                --author ${APP_AUTHOR} \
                                --version ${APP_VERSION} \
                                --icons icons/android \
                                --executables android \
                                --target ${TARGET} \
                                --libraries clipboard filewatcher pixels process skia winit webview crypto git ssl \
                                --libraries-versions ${APP_LIBRARIES_VERSIONS} \
                                --release \
                                --verbose """

                        sh "mv target/${TARGET}/release/bundle/${APP_NAME}.apk ./${APP_NAME}-${TARGET}.apk"

                        stash includes: "${APP_NAME}-${TARGET}.apk", name: "${TARGET}"
                    }
                }
                stage ('Windows x86_64') {
                    agent {
                        node {
                          label "${WINDOWS_AMD64_TARGET}-${WINDOWS_AMD64_SERVER_NAME}"
                          customWorkspace 'C:\\j\\gtvm'
                        }
                    }

                    environment {
                        TARGET = "${WINDOWS_AMD64_TARGET}"
                        LLVM_HOME = 'C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\Llvm\\x64'
                        LIBCLANG_PATH = "${LLVM_HOME}\\bin"
                        CARGO_HOME = "C:\\.cargo"
                        CARGO_PATH = "${CARGO_HOME}\\bin"
                        PATH = "${CARGO_PATH};${LIBCLANG_PATH};$PATH"
                        VM_CLIENT_EXECUTABLE = "${WORKSPACE}\\target\\${TARGET}\\release\\bundle\\${APP_NAME}\\bin\\${APP_NAME}-cli.exe"
                    }

                    steps {
                        powershell 'Remove-Item -Force -Recurse -Path target -ErrorAction Ignore'
                        powershell 'Remove-Item -Force -Recurse -Path third_party -ErrorAction Ignore'
                        powershell 'Remove-Item -Force -Recurse -Path libs -ErrorAction Ignore'

                        powershell 'git clean -fdx'
                        powershell 'git submodule foreach --recursive \'git fetch --tags\''
                        powershell 'git submodule update --init --recursive'

                        powershell "Remove-Item gtoolkit-vm-builder.exe -ErrorAction Ignore"
                        powershell "curl -o gtoolkit-vm-builder.exe https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${TARGET}.exe"

                        powershell """
                           ./gtoolkit-vm-builder.exe `
                                --app-name ${APP_NAME} `
                                --identifier ${APP_IDENTIFIER} `
                                --author ${APP_AUTHOR} `
                                --version ${APP_VERSION} `
                                --libraries ${APP_LIBRARIES} `
                                --libraries-versions ${APP_LIBRARIES_VERSIONS} `
                                --icons icons/GlamorousToolkit.ico `
                                --release `
                                --target ${TARGET} `
                                --verbose """

                        powershell "Compress-Archive -Path target/${TARGET}/release/bundle/${APP_NAME}/* -DestinationPath ${APP_NAME}-${TARGET}.zip"

                        powershell "cargo test --package vm-client-tests"
                        
                        stash includes: "${APP_NAME}-${TARGET}.zip", name: "${TARGET}"
                    }
                }
                stage ('Windows arm64') {
                    agent {
                        node {
                          label "${WINDOWS_ARM64_TARGET}-${WINDOWS_ARM64_SERVER_NAME}"
                          customWorkspace 'C:\\j\\gtvm'
                        }
                    }

                    environment {
                        TARGET = "${WINDOWS_ARM64_TARGET}"
                        HOST = "${WINDOWS_AMD64_TARGET}"
                        LLVM_HOME = 'C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\VC\\Tools\\Llvm\\x64'
                        LIBCLANG_PATH = "${LLVM_HOME}\\bin"
                        CARGO_HOME = "C:\\.cargo"
                        CARGO_PATH = "${CARGO_HOME}\\bin"
                        PATH = "${CARGO_PATH};${LIBCLANG_PATH};$PATH"
                        VM_CLIENT_EXECUTABLE = "${WORKSPACE}\\target\\${TARGET}\\release\\bundle\\${APP_NAME}\\bin\\${APP_NAME}-cli.exe"
                    }

                    steps {
                        powershell 'Remove-Item -Force -Recurse -Path target -ErrorAction Ignore'
                        powershell 'Remove-Item -Force -Recurse -Path third_party -ErrorAction Ignore'
                        powershell 'Remove-Item -Force -Recurse -Path libs -ErrorAction Ignore'

                        powershell 'git clean -fdx'
                        powershell 'git submodule foreach --recursive \'git fetch --tags\''
                        powershell 'git submodule update --init --recursive'

                        powershell "Remove-Item gtoolkit-vm-builder.exe -ErrorAction Ignore"
                        powershell "curl -o gtoolkit-vm-builder.exe https://github.com/feenkcom/gtoolkit-vm-builder/releases/download/${VM_BUILDER_VERSION}/gtoolkit-vm-builder-${HOST}.exe"

                        powershell """
                           ./gtoolkit-vm-builder.exe `
                                --app-name ${APP_NAME} `
                                --identifier ${APP_IDENTIFIER} `
                                --author ${APP_AUTHOR} `
                                --version ${APP_VERSION} `
                                --libraries boxer clipboard filewatcher gleam pixels process skia winit webview test-library crypto freetype git sdl2 ssl `
                                --libraries-versions ${APP_LIBRARIES_VERSIONS} `
                                --icons icons/GlamorousToolkit.ico `
                                --release `
                                --target ${TARGET} `
                                --verbose """

                        powershell "Compress-Archive -Path target/${TARGET}/release/bundle/${APP_NAME}/* -DestinationPath ${APP_NAME}-${TARGET}.zip"

                        stash includes: "${APP_NAME}-${TARGET}.zip", name: "${TARGET}"
                    }
                }
            }
        }
        stage ('Deployment') {
            agent {
                label "${MACOS_M1_TARGET}"
            }
            environment {
                TARGET = "${MACOS_M1_TARGET}"
            }
            when {
                expression {
                    (currentBuild.result == null || currentBuild.result == 'SUCCESS') && env.BRANCH_NAME.toString().equals('main')
                }
            }
            steps {
                unstash "${LINUX_AMD64_TARGET}"
                unstash "${LINUX_ARM64_TARGET}"
                unstash "${MACOS_INTEL_TARGET}"
                unstash "${MACOS_M1_TARGET}"
                unstash "${ANDROID_ARM64_TARGET}"
                unstash "${WINDOWS_AMD64_TARGET}"
                unstash "${WINDOWS_ARM64_TARGET}"

                sh "curl -o feenk-releaser -LsS https://github.com/feenkcom/releaser-rs/releases/download/${FEENK_RELEASER_VERSION}/feenk-releaser-${TARGET}"
                sh "chmod +x feenk-releaser"

                sh """
                ./feenk-releaser \
                    --owner ${REPOSITORY_OWNER} \
                    --repo ${REPOSITORY_NAME} \
                    --token GITHUB_TOKEN \
                    release \
                    --version ${APP_VERSION} \
                    --auto-accept \
                    --assets \
                        ${APP_NAME}-${LINUX_AMD64_TARGET}.zip \
                        ${APP_NAME}-${LINUX_ARM64_TARGET}.zip \
                        ${APP_NAME}-${MACOS_INTEL_TARGET}.app.zip \
                        ${APP_NAME}-${MACOS_M1_TARGET}.app.zip \
                        ${APP_NAME}-${ANDROID_ARM64_TARGET}.apk \
                        ${APP_NAME}-${WINDOWS_AMD64_TARGET}.zip \
                        ${APP_NAME}-${WINDOWS_ARM64_TARGET}.zip """
            }
        }
    }
}
