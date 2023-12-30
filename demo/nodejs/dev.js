const ffi = require('ffi-napi');
const ref = require('ref-napi');
const ref_struct = require('ref-struct-napi');

process.env.ZMQ_URL = 'tcp://0.0.0.0:28332';
process.env.ZMQ_TOPIC = '*';
process.env.BTC_RPC_URL = 'http://localhost:18443';
process.env.BTC_RPC_USERNAME = 'bitcoinrpc';
process.env.BTC_RPC_PASSWORD = 'bitcoinrpc';
const pathToLib = '/Users/lvcong/RustroverProjects/indexer-sdk/target/debug/libindexer_sdk.dylib';

const ByteArray = ref_struct({
    data: ref.refType(ref.types.uint8),
    length: 'size_t',
});

const rustLib = ffi.Library(pathToLib, {
    'start_processor': ['void', []],
    'js_get_event': ['char *', []],
    'free_bytes': ['void', ['char *']],
    'push_event': ['void', ['pointer', 'size_t']],
    'get_event': [ByteArray, []],
    'get_data': ['char *', ['string']],
});
rustLib.start_processor();


var height = 1;

////////////////////
const push_event_height_suffix = 0;


///////////////////
const get_data_balance_suffix = 1;


function concatTypedArrays(a, b) { // a, b TypedArray of same type
    var c = new (a.constructor)(a.length + b.length);
    c.set(a, 0);
    c.set(b, a.length);
    return c;
}

//  hex request
function get_data(request) {
    const byteDataPtr = rustLib.get_data(request);
    let ret = byteDataPtr.readCString();
    const buffer = Buffer.from(ret, 'hex');
    return buffer;
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
    height = height + 50;
    return u32ToBytes(height, push_event_height_suffix);
}

function push_event(buffer) {
    rustLib.push_event(buffer, buffer.length);
}

function handle_event(buffer) {
    if (buffer.length == 0) {
        return [];
    }
    const suffix = buffer[buffer.length - 1];
    console.log('suffix:', suffix);
    switch (suffix) {
        case 1:
            return handle_get_height();
        default:
            return [];
    }
}

function get_balance() {
    // hex:JSON(AddressTokenWrapper {
    //         address: Default::default(),
    //         token: Default::default(),
    //     }) + suffix
    var ret = get_data("7b2261646472657373223a22222c22746f6b656e223a22227d01");
    console.log('get_balance:', ret.toString());
}

get_balance();

function mainLoop() {
    // {
    //     const byteArray = rustLib.get_event();
    //     console.log('len:', byteArray.length);
    //     if (byteArray.length > 0) {
    //         // for (let i = 0; i < byteArray.length; i++) {
    //         //     console.log(`Byte at index ${i}: ${dataBuffer.readUInt8(i)}`);
    //         // }
    //         var cc=ref.reinterpretUntilZeros(byteArray.data, byteArray.length)
    //         console.log('cc:', cc);
    //         const dataBuffer = Buffer.from(cc);
    //         const uint8Array = new Uint8Array(dataBuffer);
    //         console.log('Received data:', uint8Array);
    //         const push_data = handle_event(uint8Array);
    //         if (push_data.length > 0) {
    //             push_event(push_data);
    //         }
    //     }
    // }


    {
        const byteDataPtr = rustLib.js_get_event();
        let ret = byteDataPtr.readCString();
        const buffer = Buffer.from(ret, 'hex');
        const push_data = handle_event(buffer);
        console.log("push_data:", push_data);
        if (push_data.length > 0) {
            push_event(push_data);
        }
    }

    setTimeout(mainLoop, 1000);
}
//
mainLoop();