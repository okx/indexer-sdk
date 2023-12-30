const ffi = require('ffi-napi');
const ref = require('ref-napi');
process.env.ZMQ_URL = 'tcp://0.0.0.0:28332';
process.env.ZMQ_TOPIC = '*';
const pathToLib = '/Users/lvcong/RustroverProjects/indexer-sdk/target/debug/libindexer_sdk.dylib';
const rustLib = ffi.Library(pathToLib, {
    'start_processor': ['void', []],
    'js_get_event': ['char *', []],
    'free_bytes': ['void', ['char *']],
    'push_event': ['void', ['pointer', 'size_t']],
});
rustLib.start_processor();

var height = 1;
const push_height_suffix = 0;

function concatTypedArrays(a, b) { // a, b TypedArray of same type
    var c = new (a.constructor)(a.length + b.length);
    c.set(a, 0);
    c.set(b, a.length);
    return c;
}

function u32ToBytes(u32, suffxi) {
    const buffer = new ArrayBuffer(4);
    const view = new DataView(buffer);

    var su = new Uint8Array(1);
    su[0] = suffxi;

    view.setUint32(0, u32, true);

    const uint8Array = new Uint8Array(buffer);
    return concatTypedArrays(uint8Array, su);
}

function handle_get_height() {
    height = height + 30;
    return u32ToBytes(height, push_height_suffix);
}

function push_event(buffer) {
    rustLib.push_event(buffer, buffer.length);
}

function handle_event(buffer) {
    if (buffer.length == 0) {
        return [];
    }
    const suffix = buffer[buffer.length - 1];
    switch (suffix) {
        case 1:
            return handle_get_height();
        default:
            return [];
    }
}

function mainLoop() {
    const byteDataPtr = rustLib.js_get_event();
    let ret = byteDataPtr.readCString();
    rustLib.free_bytes(byteDataPtr);
    const buffer = Buffer.from(ret, 'utf-8');
    const push_data = handle_event(buffer);
    console.log("push_data:", push_data);
    if (push_data.length > 0) {
        push_event(push_data);
    }
    // handle_event(ret);
//     const byteArray = rustLib.get_event();
//
// // Access the struct fields
//     const dataBuffer = Buffer.from(ref.address(byteArray.data), byteArray.length);
//     console.log('Data:', dataBuffer.toString('utf8'));
//
//     // push_event(byteArray);
//
    setTimeout(mainLoop, 1000);
}

//
//
mainLoop();