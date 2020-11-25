#!/bin/bash

CARGO=${HOME}/.cargo/bin/cargo

case ${ARCHS} in
    arm64*)
        echo -- BUILDING RUST COMPONENTS FOR ARM ARCHITECTURE

        ${CARGO} +nightly build -p flo_cocoa --target aarch64-apple-darwin --features cocoa --target-dir ${BUILD_DIR}/target-arm64
        
        mkdir ${BUILT_PRODUCTS_DIR}/debug
        mkdir ${BUILT_PRODUCTS_DIR}/debug/arm64
        mkdir ${BUILT_PRODUCTS_DIR}/release
        mkdir ${BUILT_PRODUCTS_DIR}/release/arm64

        if [ -e ${BUILD_DIR}/target-arm64/aarch64-apple-darwin/debug ]; then
            cp -Ra ${BUILD_DIR}/target-arm64/aarch64-apple-darwin/debug/*.a ${BUILD_DIR}/target-arm64/aarch64-apple-darwin/debug/*.dylib ${BUILT_PRODUCTS_DIR}/debug/arm64
        fi

        if [ -e ${BUILD_DIR}/target-arm64/aarch64-apple-darwin/release ]; then
            cp -Ra ${BUILD_DIR}/target-arm64/aarch64-apple-darwin/release/*.a ${BUILD_DIR}/target-arm64/aarch64-apple-darwin/release/*.dylib ${BUILT_PRODUCTS_DIR}/release/arm64
        fi
        ;;

    x86_64*)
        echo -- BUILDING RUST COMPONENTS FOR X86 ARCHITECTURE

        ${CARGO} build -p flo_cocoa --target x86_64-apple-darwin --features cocoa --target-dir ${BUILD_DIR}/target-x64
        
        mkdir ${BUILT_PRODUCTS_DIR}/debug
        mkdir ${BUILT_PRODUCTS_DIR}/debug/x64
        mkdir ${BUILT_PRODUCTS_DIR}/release
        mkdir ${BUILT_PRODUCTS_DIR}/release/x64

        if [ -e ${BUILD_DIR}/target-x64/x86_64-apple-darwin/debug ]; then
            cp -Ra ${BUILD_DIR}/target-x64/x86_64-apple-darwin/debug/*.a ${BUILD_DIR}/target-x64/x86_64-apple-darwin/debug/*.dylib ${BUILT_PRODUCTS_DIR}/debug/x64
        fi

        if [ -e ${BUILD_DIR}/target-x64/x86_64-apple-darwin/release ]; then
            cp -Ra ${BUILD_DIR}/target-x64/x86_64-apple-darwin/release/*.a ${BUILD_DIR}/target-x64/x86_64-apple-darwin/release/*.dylib ${BUILT_PRODUCTS_DIR}/release/x64
        fi
        ;;

    *)
        env | sort
        echo Unknown architecture ${ARCHS}
        exit 1
        ;;
esac
