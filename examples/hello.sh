#!/bin/bash
#
# Generated by rust-shell-script
#
. "${RUNTIME:-.}/cmd_lib.sh"

error_command() {
    do_something_failed
}

function bad_greeting() {
    local name="$1"

    info "Running error_command ..."
    error_command
    output "hello, ${name}!"
}

function good_greeting() {
    local name="$1"

    info "Running good_command ..."
    output "hello, ${name}!"
}

main() {
    local bad_ans
    bad_ans=$(_call bad_greeting "rust-shell-script")
    info "${bad_ans}"
    local good_ans
    good_ans=$(_call good_greeting "rust-shell-script")
    info "${good_ans}"
    return 0
}

main "$@"
