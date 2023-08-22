import { threads } from 'wasm-feature-detect';
import * as Comlink from 'comlink';
import * as can_parser from 'can-parser';

function wrapExports(lib) {
    return async (fileText, specText) => {
        let can_parser = new lib.CANParserWasm(
            "Warn", 
            "^\\((?P<timestamp>[0-9]+\\.[0-9]+)\\).*?(?P<id>[0-9A-F]{3,8})#(?P<data>[0-9A-F]+)", 
            { "J1939": specText }  // replace the hardcoded path with specText
        );
        can_parser.parse_lines(fileText);
        return "Done";
    };
}

async function initHandlers() {
    const startParser = async () => {
        const lib = await import('can-parser');
        await lib.default();
        console.log("Hardware Concurrency", navigator.hardwareConcurrency);
        // await lib.initThreadPool(navigator.hardwareConcurrency); 
        return wrapExports(lib);
    }

    let parserStarted = await startParser();

    return Comlink.proxy({
        parser: parserStarted
    });
}

Comlink.expose({
    handlers: initHandlers()
});
