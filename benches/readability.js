import fs from 'fs';
import jsdom from 'jsdom';
import readability from '@mozilla/readability';

const { JSDOM } = jsdom;
const { Readability } = readability;

const benchReadability = (filePath) => {
    const html = fs.readFileSync(filePath, 'utf-8');
    const dom = new JSDOM(html);
    
    console.time('readability.js');
    const reader = new Readability(dom.window.document);
    const article = reader.parse();
    console.timeEnd('readability.js');
};

benchReadability('benches/hn.html');
