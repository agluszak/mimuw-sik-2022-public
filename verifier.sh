#!/usr/bin/env bash

function check_executable() {
  EXECUTABLE=$1
  if [ -f "$EXECUTABLE" ]; then
    echo "$EXECUTABLE executable found"
    ANYTHING_BUILT=true

    ./"$EXECUTABLE" >/dev/null 2>&1
    if [ $? -ne 1 ]; then
      echo "$EXECUTABLE should fail with exit code 1 when no arguments are given"
    fi

    OUTPUT=$(mktemp)
    ./"$EXECUTABLE" -h >"$OUTPUT" 2>/dev/null
    if [ $? -ne 0 ]; then
      echo "$EXECUTABLE should exit with exit code 0 when -h is given"
    fi

    # Check if output is non-empty
    if [ -s "$OUTPUT" ]; then
      echo "$EXECUTABLE should output help text to stdout when -h is given"
    fi
  fi
}

function verify() {
  if [ -z "$1" ]; then
    echo "Usage: $0 <file>"
    exit 1
  fi
  if [ ! -f "$1" ]; then
    echo "File $1 does not exist"
    exit 1
  fi
  if [ ! -r "$1" ]; then
    echo "File $1 is not readable"
    exit 1
  fi
  if [ ! -s "$1" ]; then
    echo "File $1 is empty"
    exit 1
  fi

  # Check if file is a valid zip file
  unzip -t "$1" >/dev/null
  if [ $? -ne 0 ]; then
    echo "File $1 is not a valid zip file"
    exit 1
  fi

  # Create a temp directory
  TEMP_DIR=$(mktemp -d)

  # Unzip to the temp directory
  unzip -q "$1" -d "$TEMP_DIR"
  pushd "$TEMP_DIR" >/dev/null || exit 1

  SERVER_EXECUTABLE="robots-server"
  CLIENT_EXECUTABLE="robots-client"

  # Check if cmake works
  cmake . >/dev/null 2>&1
  if [ $? -ne 0 ]; then
    echo "CMake failed, trying make"
  fi

  ALL_OK=true

  # Check if there is a makefile

  ls | grep -i "makefile" >/dev/null 2>&1
  if [ $? -eq 0 ]; then
    TEMP_FILE=$(mktemp)
    # Check if the solution can be built using `make`
    make SHELL="/bin/bash -x" >"$TEMP_FILE" 2>&1
    STATUS=$?

    REQUIRED_FLAGS=("-Wall" "-Wextra" "-Wconversion" "-Werror" "-O2")
    STANDARD_FLAG=("-std=gnu++20" "-std=gnu17")

    for flag in "${REQUIRED_FLAGS[@]}"; do
      if ! grep -q -- "$flag" "$TEMP_FILE"; then
        echo "Missing flag $flag"
        ALL_OK=false
      fi
    done

    FOUND_STANDARD_FLAG=false
    for flag in "${STANDARD_FLAG[@]}"; do
      if grep -q -- "$flag" "$TEMP_FILE"; then
        FOUND_STANDARD_FLAG=true
      fi
    done

    if [ "$FOUND_STANDARD_FLAG" = false ]; then
      echo "Missing flag -std=gnu17 or -std=gnu++20"
      ALL_OK=false
    fi

    # Check if there were warnings or errors during build
    grep "note:" "$TEMP_FILE"
    if [ $? -eq 0 ]; then
      echo "Notes during build"
      ALL_OK=false
    fi

    grep "warning:" "$TEMP_FILE"
    if [ $? -eq 0 ]; then
      echo "Warnings during build"
      ALL_OK=false
    fi

    grep "error:" "$TEMP_FILE"
    if [ $? -eq 0 ]; then
      echo "Errors during build"
      ALL_OK=false
    fi

    if [ $STATUS -eq 0 ]; then
      ANYTHING_BUILT=false
      check_executable "$SERVER_EXECUTABLE"
      check_executable "$CLIENT_EXECUTABLE"

      if [ "$ANYTHING_BUILT" = false ]; then
        echo "Solution was built, but no executable was found. Are the names correct?"
        ALL_OK=false
      fi
    else
      echo "Solution could not be built"
      ALL_OK=false
    fi
  else
    echo "No makefile found. Is the solution correctly packed?"
    ALL_OK=false
  fi

  popd >/dev/null || exit 1
  if [ "$ALL_OK" = true ]; then
    echo "Solution is valid"
    exit 0
  else
    echo "Solution is not valid"
    exit 1
  fi
}

verify "$@"
