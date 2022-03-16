#!/usr/bin/env bash

arguments=$*
if [ $# -eq 0 ]
  then
    arguments="--interactive"
fi

if [[ "$OSTYPE" == "linux-gnu"* ]]; then
  ./../target/x86_64-unknown-linux-gnu/release/bundle/bin/GlamorousToolkit-cli GlamorousToolkit.image $arguments
elif [[ "$OSTYPE" == "darwin"* ]]; then
  arch_name="$(uname -m)"
  is_m1=false
  if [ "${arch_name}" = "x86_64" ]; then
      if [ "$(sysctl -in sysctl.proc_translated)" = "1" ]; then
          is_m1=true
      fi
  elif [ "${arch_name}" = "arm64" ]; then
      is_m1=true
  fi

  if [[ "$is_m1" == true ]]; then
    ./../target/aarch64-apple-darwin/release/bundle/GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit-cli GlamorousToolkit.image $arguments
  else
    ./../target/x86_64-apple-darwin/release/bundle/GlamorousToolkit.app/Contents/MacOS/GlamorousToolkit-cli GlamorousToolkit.image $arguments
  fi

elif [[ "$OSTYPE" == "cygwin" || "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
  echo "$OSTYPE is unsupported. Please submit an issue at https://github.com/feenkcom/gtoolkit/issues".
  exit 1
else
  echo "$OSTYPE is unsupported. Please submit an issue at https://github.com/feenkcom/gtoolkit/issues".
  exit 1
fi

exit 0