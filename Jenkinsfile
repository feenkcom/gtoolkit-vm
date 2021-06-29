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
        APP_NAME = "GlamorousToolkit"

        MACOS_INTEL_TARGET = 'x86_64-apple-darwin'
        MACOS_M1_TARGET = 'aarch64-apple-darwin'
        WINDOWS_AMD64_TARGET = 'x86_64-pc-windows-msvc'
        LINUX_AMD64_TARGET = 'x86_64-unknown-linux-gnu'
    }

    stages {
        stage ('Parallel build') {
            parallel {
                stage ('MacOS x86_64') {
                    agent {
                        label "${MACOS_INTEL_TARGET}"
                    }

                    environment {
                        TARGET = "${MACOS_INTEL_TARGET}"
                        PATH = "$HOME/.cargo/bin:/usr/local/bin/:$PATH"
                    }

                    steps {
                        sh 'git clean -fdx'
                        sh 'git submodule update --init --recursive'

                        //sh "cargo run --package vm-builder --target ${TARGET} -- --app-name ${APP_NAME} -vv --release"
                        sh "ditto -c -k --sequesterRsrc --keepParent target/${TARGET}/release/bundle/${APP_NAME}.app ${APP_NAME}${TARGET}.app.zip"

                        stash includes: "${APP_NAME}${TARGET}.app.zip", name: "${TARGET}"
                    }
                }
                stage ('MacOS M1') {
                    agent {
                        label "${MACOS_M1_TARGET}"
                    }

                    environment {
                        TARGET = "${MACOS_M1_TARGET}"
                        PATH = "$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
                    }

                    steps {
                        sh 'git clean -fdx'
                        sh 'git submodule update --init --recursive'

                        //sh "cargo run --package vm-builder --target ${TARGET} -- --app-name ${APP_NAME} -vv --release"
                        sh "ditto -c -k --sequesterRsrc --keepParent target/${TARGET}/release/bundle/${APP_NAME}.app ${APP_NAME}${TARGET}.app.zip"

                        stash includes: "${APP_NAME}${TARGET}.app.zip", name: "${TARGET}"
                    }
                }
                stage ('Linux x86_64') {
                    agent {
                        label "${LINUX_AMD64_TARGET}"
                    }
                    environment {
                        TARGET = "${LINUX_AMD64_TARGET}"
                        PATH = "$HOME/.cargo/bin:$PATH"
                    }

                    steps {
                        sh 'git clean -fdx'
                        sh 'git submodule update --init --recursive'

                        //sh "cargo run --package vm-builder --target ${TARGET} -- --app-name ${APP_NAME} -vv --release"

                        sh """
                            cd target/${TARGET}/release/bundle/
                            zip -r ${APP_NAME}${TARGET}.zip ./${APP_NAME}/
                            """

                        sh 'mv target/${TARGET}/release/bundle/${APP_NAME}${TARGET}.zip ./${APP_NAME}${TARGET}.zip'

                        stash includes: "${APP_NAME}${TARGET}.zip", name: "${TARGET}"
                    }
                }
                stage ('Windows x86_64') {
                    agent {
                        label "${WINDOWS_AMD64_TARGET}"
                    }

                    environment {
                        TARGET = "${WINDOWS_AMD64_TARGET}"
                        LIBCLANG_PATH = "C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\VC\\Tools\\Llvm\\x64\bin"
                        PATH = "${homedrive}${homepath}\\.cargo\\bin;C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\MSBuild\\Current\\Bin;C:\\Program Files\\CMake\\bin;$PATH"
                    }

                    steps {
                        powershell 'echo $env:path'
                        powershell 'git clean -fdx'
                        powershell 'git submodule update --init --recursive'

                        powershell "cargo run --package vm-builder --target ${TARGET} -- --app-name ${APP_NAME} -vv --release"

                        //powershell "New-Item -path target/${TARGET}/release/bundle/${APP_NAME}/bin -type directory"
                        powershell "Compress-Archive -Path target/${TARGET}/release/bundle/${APP_NAME} -DestinationPath ${APP_NAME}${TARGET}.zip"
                        stash includes: "${APP_NAME}${TARGET}.zip", name: "${TARGET}"
                    }
                }
            }
        }

        stage ('Deployment') {
            agent {
                label "unix"
            }
            environment {
                PATH = "$HOME/.cargo/bin:$PATH"
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

                sh """
                cargo run --package vm-releaser -- \
                    --owner feenkcom \
                    --repo gtoolkit-vm \
                    --token GITHUB_TOKEN \
                    --bump-patch \
                    --auto-accept \
                    --assets \
                        ${APP_NAME}${LINUX_AMD64_TARGET}.zip \
                        ${APP_NAME}${MACOS_INTEL_TARGET}.app.zip \
                        ${APP_NAME}${MACOS_M1_TARGET}.app.zip \
                        ${APP_NAME}${WINDOWS_AMD64_TARGET}.zip """
            }
        }
    }
}
