import init, {ConvertMain} from '../pkg/all_in_one_phonetic.js';
import wasmModule from '../pkg/all_in_one_phonetic_bg.wasm'; // Direct import, no bindings
import index_html from '../index.html'

let wasmInitialized = false;
let converterInstance;

// Ensure WASM is initialized
async function ensureWasmInitialized() {
    if (!wasmInitialized) {
        await init(wasmModule);
        wasmInitialized = true;
    }
}

async function handleRequest(request) {
    const { pathname } = new URL(request.url);

    const headers = {
        'Access-Control-Allow-Origin': '*', // Allow all origins, or specify your domain like 'http://localhost:5000'
        'Access-Control-Allow-Methods': 'POST, OPTIONS', // Allow specific methods
        'Access-Control-Allow-Headers': 'Content-Type, Authorization', // Allow specific headers
        'Access-Control-Allow-Credentials': 'true' // Allow credentials if needed
    };

    // Handle OPTIONS preflight request (browser sends this before actual request)
    if (request.method === 'OPTIONS') {
        return new Response(null, {
            status: 204, // No content
            headers: headers
        });
    }

    // Handle POST request for phonetic conversion
    if (pathname === '/api/convert' && request.method === 'POST') {
        try {
            const body = await request.json();
            const text = body.text;
            const operation = body.operation;  // Assuming `operation` is passed from the client

            if (!text || !operation) {
                return new Response(
                    JSON.stringify({ error: 'Missing \'text\' or \'operation\' field' }),
                    {
                        status: 400,
                        headers: { 'Content-Type': 'application/json', ...headers }
                    }
                );
            }

            const result = await convertPhonetic(text, operation);

            return new Response(
                JSON.stringify(result),
                {
                    status: 200,
                    headers: { 'Content-Type': 'application/json', ...headers }
                }
            );
        } catch (err) {
            return new Response(
                JSON.stringify({ error: 'Failed to process the request', details: err.message }),
                {
                    status: 500,
                    headers: { 'Content-Type': 'application/json', ...headers }
                }
            );
        }
    }

    // Default return for any other request
    return new Response(index_html, {
        status: 200,
        headers: {
            "Content-Type": "text/html",
            "Access-Control-Allow-Origin": "*",
        },
    });
}

// Convert text to phonetic using the initialized WASM converter
async function convertPhonetic(text, operation) {
    await ensureWasmInitialized();

    // Using the existing singleton converterInstance
    const requestData = {
        text: text,
        operation: operation // assuming the operation is either 'MapRes' or 'SentRes'
    };

    try {
        // Call ConvertMain to process the phonetic conversion
        return await ConvertMain(requestData);
    } catch (err) {
        throw new Error("Failed to convert text to phonetic: " + err.message);
    }
}

//server listener
addEventListener('fetch', event => {
    event.respondWith(handleRequest(event.request));
})
// Export the Worker fetch handler
export default {
    fetch: handleRequest
};
