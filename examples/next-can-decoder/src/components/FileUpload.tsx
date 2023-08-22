import React, { useEffect, useState } from 'react';
import { wrap } from 'comlink';

type FileUploaderProps = {
    specFile: File | null;
}

const FileUploader: React.FC = ({ specFile }) => {
    const [parseFile, setParseFile] = useState<Function | null>(null);
    const [result, setResult] = useState<string | null>(null);

    useEffect(() => {
        if (specFile) {
            const setupWorker = async () => {
                const worker = new Worker(new URL('../workers/wasmWorker.js', import.meta.url));
                const handler = wrap(worker);
                const { parser } = await handler.handlers;
                setParseFile(() => parser);  // Store the parser function from the worker
            };
            setupWorker();
        }
    }, [specFile]);

    const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (file && parseFile && specFile) {
            const text = await file.text();
            let lines = text.split('\n');
            const specText = await specFile.text(); 
            const parsedResult = await parseFile(lines, specText);
            setResult("wooo!");
        }
    };

    return (
        <div>
            <input type="file" onChange={handleFileChange} />
            <div>{result}</div>
        </div>
    );
}

export default FileUploader;
