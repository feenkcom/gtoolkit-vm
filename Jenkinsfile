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
    }

    stages {
        stage ('Parallel build') {
            parallel {
                stage ('MacOSX') {
                    agent {
                        label "macosx" // a labelled node defined in jenkins
                    }

                    environment {
                        TARGET = 'x86_64-apple-darwin'
                    }

                    steps {
                        sh 'git clean -fdx'

                        // the .app is in ./target/x86_64-apple-darwin/release/bundle/GlamorousToolkit.app
                        //sh 'cargo run --package vm-builder --target ${env.TARGET} -- --app-name GlamorousToolkit -vv --release'

                        sh 'mkdir -p target/${TARGET}/release/bundle/GlamorousToolkit.app'

                        stash includes: 'target/${TARGET}/release/bundle/GlamorousToolkit.app', name: '${TARGET}'
                    }
                }
                stage ('Windows') {
                    agent {
                        label "windows" // a labelled node defined in jenkins
                    }
                }
            }
        }
    }
}
