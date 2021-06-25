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
    }

    stages {
        stage ('Parallel build') {
            parallel {
                stage ('MacOSX') {
                    agent {
                        label "macosx" // a labelled node defined in jenkins
                    }

                    environment {
                        TARGET = "${MACOS_INTEL_TARGET}"
                    }

                    steps {
                        sh 'git clean -fdx'

                        //sh "cargo run --package vm-builder --target ${env.TARGET} -- --app-name ${APP_NAME} -vv --release"

                        sh "mkdir -p target/${TARGET}/release/bundle/${APP_NAME}.app"
                        sh "zip ${APP_NAME}${TARGET}.app.zip target/${TARGET}/release/bundle/${APP_NAME}.app"

                        stash includes: "${APP_NAME}${TARGET}.app.zip", name: "${TARGET}"
                    }
                }
                stage ('Windows') {
                    agent {
                        label "windows" // a labelled node defined in jenkins
                    }

                    steps {
                        powershell 'echo $PATH'
                    }
                }
            }
        }

        stage ('Deployment') {
            agent {
                label "unix"
            }
            steps {
                unstash "${MACOS_INTEL_TARGET}"
                sh "unzip ${APP_NAME}${MACOS_INTEL_TARGET}.app.zip -d ${APP_NAME}${MACOS_INTEL_TARGET}"

                sh 'ls -la'
                sh 'ls -la ${APP_NAME}${MACOS_INTEL_TARGET}'

                //sh "cargo run --package vm-releaser -- --owner feenkcom --repo ${APP_NAME} --token GITHUB_TOKEN --bump-patch --auto-accept --assets"

            }
        }
    }
}
