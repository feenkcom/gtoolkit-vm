import hudson.tasks.test.AbstractTestResultAction
import hudson.model.Actionable
import hudson.tasks.junit.CaseResult

pipeline {
    agent none
    parameters { string(name: 'FORCED_TAG_NAME', defaultValue: '', description: 'Environment variable used for increasing the minor or major version of Glamorous Toolkit. Example input `v0.8.0`. Can be left blank, in that case, the patch will be incremented.') }
    options {
        buildDiscarder(logRotator(numToKeepStr: '50'))
        disableConcurrentBuilds()
    }
    environment {
        GITHUB_TOKEN = credentials('githubrelease')
        AWSIP = 'ec2-18-197-145-81.eu-central-1.compute.amazonaws.com'
        MASTER_WORKSPACE = ""
        APP_NAME = 'GlamorousToolkit'
        APP_IDENTIFIER = 'com.gtoolkit'
        APP_LIBRARIES = 'git sdl2 boxer clipboard gleam glutin skia'
        APP_AUTHOR = '"feenk gmbh <contact@feenk.com>"'

        LINUX_SERVER_NAME = 'mickey-mouse'
        MACOS_INTEL_TARGET = 'x86_64-apple-darwin'
        MACOS_M1_TARGET = 'aarch64-apple-darwin'

        WINDOWS_SERVER_NAME = 'daffy-duck'
        WINDOWS_AMD64_TARGET = 'x86_64-pc-windows-msvc'
        LINUX_AMD64_TARGET = 'x86_64-unknown-linux-gnu'
    }

    stages {
        stage('Run CI?') {
            agent any
            steps {
                script {
                    if (sh(script: "git log -1 --pretty=%B | fgrep -ie '[skip ci]' -e '[ci skip]'", returnStatus: true) == 0) {
                        currentBuild.result = 'NOT_BUILT'
                        success 'Aborting because commit message contains [skip ci]'
                    }
                }
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
                        sh 'git clean -fdx'
                        sh 'git submodule update --init --recursive'

                        sh """
                            cargo run --package vm-builder --target ${TARGET} --release -- \
                                --vmmaker-vm /Users/tudor/vmmaker/Pharo.app/Contents/MacOS/Pharo \
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
                        sh 'git clean -fdx'
                        sh 'git submodule update --init --recursive'

                        sh """
                            cargo run --package vm-builder --target ${TARGET} --release -- \
                                --vmmaker-vm /Users/tudor/vmmaker/Pharo.app/Contents/MacOS/Pharo \
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
                        sh 'git clean -fdx'
                        sh 'git submodule update --init --recursive'

                        sh """
                            cargo run --package vm-builder --target ${TARGET} --release -- \
                                --vmmaker-vm /home/ubuntu/jenkins/vmmaker/pharo \
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
                        powershell 'git clean -fdx'
                        powershell 'git submodule update --init --recursive'

                        powershell """
                           cargo run --package vm-builder --target ${TARGET} --release -- `
                                --vmmaker-vm C:/j/vmmaker/PharoConsole.exe `
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
