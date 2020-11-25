#!/bin/bash

CARGO=${HOME}/.cargo/bin/cargo

case ${ARCHS} in
	arm64*)
		${CARGO} +nightly build -p flo_cocoa --target aarch64-apple-darwin --features cocoa --target-dir target-arm64
        mkdir target
        mkdir target/debug
        cp -Ra target-arm64/aarch64-apple-darwin/debug target/debug
		;;

    x86_64*)
        ${CARGO} build -p flo_cocoa --target x86_64-apple-darwin --features cocoa --target-dir target-x86
        mkdir target
        mkdir target/debug
        cp -Ra target-x86/x86_64-apple-darwin/debug target/debug
        ;;

    *)
        env | sort
        echo Unknown architecture ${ARCHS}
        exit 1
        ;;
esac
