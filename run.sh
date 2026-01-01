#!/usr/bin/env bash

function testdata() {
	cargo run -- data/2632.kml
}

function compute() {
	cargo run -- "$@"
}

function test() {
	cargo test "$@"
}

function init() {
	export RUST_LOG=trace
	#export RUST_LOG=
}

function main() {
	local F="$1"
	shift
	$F "$@"
}

init
main "$@"
