import typescript from '@rollup/plugin-typescript';
import resolve from '@rollup/plugin-node-resolve';
import commonjs from '@rollup/plugin-commonjs';

import { readFileSync } from 'fs';

function htmlPlugin() {
    return {
        name: 'html-loader',
        transform(code, id) {
            if (id.endsWith('.html')) {

                return {
                    code: `export default ${JSON.stringify(code)}`,
                    map: null
                };
            }
        }
    };
}
export default {
    input: 'temp/index.ts',
    output: {
        file: 'out/index.js',
        format: 'es',
    },
    external: ['runtimebindings'],
    plugins: [
        htmlPlugin(),
        typescript(),
        commonjs(),
        resolve(),
    ],
};
