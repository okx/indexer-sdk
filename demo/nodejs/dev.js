// process.env.ZMQ_URL = 'tcp://127.0.0.1:5555';
// process.env.ZMQ_TOPIC = '*';
//
// function sleep(ms) {
//     return new Promise(resolve => setTimeout(resolve, ms))
// }
//
// //
// const ffi = require('ffi-napi');
//
// const pathToLib = '/Users/lvcong/RustroverProjects/indexer-sdk/target/debug/libindexer_sdk.dylib';
//
// const myLib = ffi.Library(pathToLib, {
//     'start_processor': ['void',[]]
// });
//
// async function main() {
//     console.log(1)
//     myLib.start_processor();
//     await sleep(1000000)
//     console.log(2)
// }
//
// main()


const ffi = require('ffi-napi');
const ref = require('ref-napi');

const pathToLib = '/Users/lvcong/RustroverProjects/indexer-sdk/target/debug/libindexer_sdk.dylib';
const rustLib = ffi.Library(pathToLib, {
    'start_processor': ['void', []],
    'get_event': ['pointer', []],
});

// 定义 Node.js 函数
function push_event(data) {
    // Do something with the data (dummy example: print the data)
    console.log('Pushing event:', data);
}
process.env.ZMQ_URL = 'tcp://0.0.0.0:28332';
process.env.ZMQ_TOPIC = '*';
// 主循环
function mainLoop() {
    rustLib.start_processor();
    // 调用 Rust 函数 get_event
    const eventData = rustLib.get_event();

    const buffer = Buffer.from(ref.reinterpret(eventData, eventData.length, 0));
    console.log("receive event data: ", buffer.toString());

    // 如果有字节数组，则调用 Rust 函数 execute
    push_event(buffer);

    // 释放 Rust 返回的指针

    // 间隔 1 秒后再次调用主循环
    setTimeout(mainLoop, 1000);
}

// 启动主循环
mainLoop();