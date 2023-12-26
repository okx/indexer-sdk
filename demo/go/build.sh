#!/bin/sh

go build -o go-rust  -ldflags="-r /Users/lvcong/RustroverProjects/indexer-sdk/target/debug" main.go