package main

/*
#cgo CFLAGS: -I/Users/lvcong/RustroverProjects/indexer-sdk/target/debug
#cgo LDFLAGS: -L/Users/lvcong/RustroverProjects/indexer-sdk/target/debug -lmylibrary
#include "./indexersdk.h"
#include <stdlib.h>
*/
import "C"
import (
	"fmt"
	"os"
	"time"
	"unsafe"
)

func main() {
	err := os.Setenv("ZMQ_URL", "tcp://0.0.0.0:28332")
	if nil != err {
		println(err)
		panic(err)
	}
	err = os.Setenv("ZMQ_TOPIC", "*")
	if nil != err {
		println(err)
		panic(err)
	}
	println(1)
	C.start_processor()
	println(2)
	time.Sleep(time.Second * 10)

	// 调用 Rust 函数
	rustByteArray := C.get_data()
	// 将 Rust 结构体映射到 Go 结构体
	goByteArray := ByteArray{
		Data:   rustByteArray.data,
		Length: rustByteArray.length,
	}
	// 将 C 字节数组转换为 Go 字节数组
	goData := C.GoBytes(unsafe.Pointer(goByteArray.Data), C.int(goByteArray.Length))

	// 输出结果
	fmt.Printf("Data: %s\n", goData)
	fmt.Printf("Length: %d\n", goByteArray.Length)
	println(3)

	time.Sleep(time.Second * 1000000)
}

type ByteArray struct {
	Data   *C.uchar
	Length C.size_t
}
