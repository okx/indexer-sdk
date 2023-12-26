process.env.ZMQ_URL = 'tcp://127.0.0.1:5555';
process.env.ZMQ_TOPIC = '*';

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms))
}

//
const ffi = require('ffi-napi');

const pathToLib = '/Users/lvcong/RustroverProjects/indexer-sdk/target/debug/libmylibrary.dylib';

const myLib = ffi.Library(pathToLib, {
    'start_processor': ['void']
});

// 调用函数
async function main() {
    console.log(1)
    myLib.start_processor();
    await sleep(1500)
    console.log(2)
}

main()